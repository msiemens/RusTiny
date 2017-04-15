//! AST -> IR Translation
//!
//! This module translates the tree-like AST representation into the more linear
//! intermediate representation (IR). In the process, the code is made more
//! explicit. This includes the evaluation order, the storage location of
//! temporary values and (conditional) branch information.
//!
//! # Register-to-register IR
//!
//! As described in *Engineering a Compiler*, there are two types of intermediate
//! representations: memory-to-memory and register-to-register. The former assumes
//! all instructions to operate on memory locations. In this case the register
//! allocator is just an optimization to minimize memory access.
//! On the other hand, register-to-register IRs assume an infinite number of
//! registers. Also, instructions operate ONLY on registers! Here the register
//! allocator is indispensable because the target architecture (usually) doesn't
//! provide enough registers.
//! In this compiler I've choosen the register-to-register model as both
//! *Introduction to Compiler Design*, *Engineering a Compiler* and the LLVM IR
//! use it too.
//!
//! # Storage locations/choice of registers
//!
//! As stated in *Introduction to Compiler Design*, there are two approaches to
//! translating expressions:
//!
//! 1. Each sub-expression chooses a storage location and passes it up to the
//!    parent.
//! 2. The parent determines a storage location for its children's values.
//!
//! In *Introduction to Compiler Design* the first approach is preferred because
//! sub-expressions might alter some values needed by the parent (e.g.
//! `x + (x = 2)` (?). I think such behaviour it not possible in `RusTiny`.
//! Therefore, I might rewrite the translator to use the first approach instead.
//!
//! # Implementation notes
//!
//! Each translation function takes a mutable reference to the current block
//! (except for `translate_fn` which creates the block). That way we can commit
//! the current block and continue with a new block in the middle of a function
//! (see `commit_block_and_continue`). I think passing the pointer along is better
//! than getting a reference to the current block every time we need it using a
//! function.

// TODO: SSA verifier?

use std::collections::{HashMap, HashSet, VecDeque};
use std::mem;
use driver::interner::Ident;
use driver::session;
use front::ast;
use middle::ir::{self, Register};
use front::ast::visit::*;


mod controlflow;
mod expr;


/// Information about the function that's translated currently
struct FunctionContext {
    /// The generated function body
    body: Vec<ir::Block>,

    /// The function's arguments
    args: Vec<Ident>,

    /// All registers used in this function
    registers: HashSet<Register>,

    /// The current block's scope
    scope: ast::NodeId,

    /// A return slot used to store the return value
    /// if the return type is non-void
    return_slot: Option<Register>,

    /// The next free register to use
    next_register: u32,

    /// The current loop's exit
    loop_exit: Option<ir::Label>,
}


/// The destination of a computation
///
/// Sometimes we want an expression to store its result in a specific register.
/// This is expressed by `Dest::Store(..)`.
/// But sometimes we just want the computation and don't care about where the
/// the result is stored. This behaviour can be requested by using `Dest::Ignore`
#[derive(Clone, Copy, Debug)]
pub enum Dest {
    Store(Register),
    Ignore
}


/// The AST -> IR translator
pub struct Translator {
    ir: ir::Program,
    fcx: Option<FunctionContext>,
    /// As the translator might want to use the same label multiple times,
    /// we always append an index, which is stored here, to it
    next_label: HashMap<Ident, u32>,
}

impl Translator {
    pub fn new() -> Translator {
        Translator {
            ir: ir::Program::new(),
            fcx: None,
            next_label: HashMap::new(),
        }
    }

    /// Access the current function context
    ///
    /// # Panics
    ///
    /// Panics when we're not translating a function at the moment.
    ///
    fn fcx(&mut self) -> &mut FunctionContext {
        self.fcx.as_mut().unwrap()
    }

    /// Get the next free register
    fn next_free_register(&mut self) -> Register {
        let next_register = self.fcx().next_register;
        self.fcx().next_register += 1;

        Register::from_str(&next_register.to_string())
    }

    /// Get the next free label with a given base name (e.g. `id` -> `id2`)
    fn next_free_label(&mut self, basename: Ident) -> ir::Label {
//        let next_label = &mut self.fcx().next_label;

        *self.next_label.entry(basename).or_insert(0) += 1;

        let mut name = basename.to_string();
        name.push_str(&self.next_label[&basename].to_string());
        ir::Label::from_str(&name)
    }

    /// Unwrap a destination
    ///
    /// When the destination contains a register, we return it. Otherwise we
    /// take the next free register.
    fn unwrap_dest(&mut self, dest: Dest) -> Register {
        match dest {
            Dest::Store(r) => r,
            Dest::Ignore => self.next_free_register()
        }
    }

    fn with_first_block<F>(&mut self, current: &mut ir::Block, f: F) where F: Fn(&mut ir::Block) -> () {
        match self.fcx().body.first_mut() {
            Some(first) => f(first),
            None => f(current)
        };
    }

    /// Commit the current block and continue working with an empty one
    ///
    /// *Note:* To create a new block we have to give it a label. For that reason
    /// we take a label here.
    fn commit_block_and_continue(&mut self, block: &mut ir::Block, label: ir::Label) {
        // Make sure the current block is finalized
        assert_ne!(block.last, ir::ControlFlowInstruction::NotYetProcessed);

        let mut new_block = ir::Block {
            label: label,
            inst: VecDeque::new(),
            last: ir::ControlFlowInstruction::NotYetProcessed,
            phis: Vec::new(),
        };

        mem::swap(block, &mut new_block);

        // We need to push `new_block` here as it's contains `block` after the
        // swap.
        self.fcx().body.push(new_block);
    }

    /// Commit the current block
    fn commit_block(&mut self, block: ir::Block) {
        self.fcx().body.push(block);
    }

    /// Register a local variable and return its register
    fn register_local(&mut self, id: Ident) -> Register {
        // Find a register
        let mut id_mangled = id;
        let mut i = 1;

        while self.fcx().registers.contains(&Register(id_mangled)) {
            id_mangled = Ident::from_str(&format!("{}{}", id, i));
            i += 1;
        }

        // Register the variable
        let register = Register(id_mangled);
        self.fcx().registers.insert(register);

        let sytable = &session().symbol_table;
        sytable.set_register(self.fcx().scope, &id, register);

        register
    }

    fn lookup_register(&mut self, name: &Ident) -> Register {
        let sytable = &session().symbol_table;
        let var = sytable.resolve_variable(self.fcx().scope, name)
            .expect(&format!("variable {} not yet declared", name));
        var.reg.expect(&format!("{:?} is not a local variable", var))
    }

    /// Translate a function
    fn trans_fn(&mut self,
                name: Ident,
                bindings: &[ast::Binding],
                ret_ty: ast::Type,
                body: &ast::Node<ast::Block>) {
        let is_void = ret_ty == ast::Type::Unit;

        // Prepare function context
        let ret_slot = Register::from_str("ret_slot");
        self.fcx = Some(FunctionContext {
            body: Vec::new(),
            args: bindings.iter().map(|binding| *binding.name).collect(),
            registers: HashSet::new(),
            return_slot: if is_void { None } else { Some(ret_slot) },
            scope: body.id,
            next_register: 0,
            loop_exit: None
        });

        // Prepare ast block
        let mut block = ir::Block {
            label: self.next_free_label(Ident::from_str("entry-block")),
            inst: VecDeque::new(),
            last: ir::ControlFlowInstruction::NotYetProcessed,
            phis: Vec::new(),
        };

        // PREVIOUSLY: Allocate arguments (from left to right)

        // Register return slot as register variable
        //        self.register_local(ret_slot.ident());

        // Register arguments as local variables so translation works
        // FIXME: Better solution? We seem to confuse stack slots with local variables here
        for binding in bindings.iter().rev() {
            self.register_local(*binding.name);
        };

        // Translate ast block
        if is_void {
            self.trans_block(body, &mut block, Dest::Ignore);
        } else {
            // Store block value in temporary register...
            // NOTE: If the block contains a return expression, the ret_value register we pass
            // is skipped.
            let ret_value = self.next_free_register();
            self.trans_block(body, &mut block, Dest::Store(ret_value));
            // ... and store it in the memory slot
            if !block.finalized() {
                // If the block contains a 'return' expr, it already stores the result in the
                // return slot, so we don't have to do
                block.store(ir::Value::Register(ret_value), ir::Value::Register(ret_slot));
            };
        };

        // Finalize the function
        if is_void && !block.finalized() {
            block.ret(None);
        } else {
            // Build the return block
            // FIXME: If there is a single store to the return slot,
            //        return it directly and skip the alloca/store
            self.with_first_block(&mut block, |block| block.alloc(ret_slot));

            let return_label = self.next_free_label(Ident::from_str("return"));
            if !block.finalized() {
                block.jump(return_label);
            };

            // %reg = mem[%return_slot]
            // ret %reg
            self.commit_block_and_continue(&mut block, return_label);
            let return_value = self.next_free_register();
            block.load(ir::Value::Register(self.fcx().return_slot.unwrap()), return_value);
            block.ret(Some(ir::Value::Register(return_value)));
        };

        self.commit_block(block);

        // Emit the symbol
        let fcx = self.fcx.take().unwrap();
        self.ir.emit(ir::Symbol::Function {
            name: name,
            body: fcx.body,
            args: bindings.iter().map(|b| *b.name).collect(),
        });
    }

    /// Translate an AST code block
    fn trans_block(&mut self,
                   b: &ast::Node<ast::Block>,
                   block: &mut ir::Block,
                   dest: Dest) {
        with_reset!(self.fcx().scope, b.id, {
            for stmt in &b.stmts {
                self.trans_stmt(stmt, block);
            }

            self.trans_expr(&b.expr, block, dest);
        });
    }

    /// Translate a statement
    fn trans_stmt(&mut self,
                  stmt: &ast::Statement,
                  block: &mut ir::Block) {
        match *stmt {
            ast::Statement::Declaration { ref binding, ref value } => {
                // Allocate memory on stack for the binding
                let dst = self.register_local(*binding.name);
                self.with_first_block(block, |block| block.alloc(dst));

                // Store the expression in the new slot
                let value = self.trans_expr_to_value(value, block);
                block.store_reg(value, dst);
            },
            ast::Statement::Expression { ref val } => {
                // We don't care where the value of the expression is stored,
                // thus `Dest::Ignore`.
                self.trans_expr(val, block, Dest::Ignore);
            }
        }
    }
}


impl<'v> Visitor<'v> for Translator {
    fn visit_symbol(&mut self, s: &'v ast::Node<ast::Symbol>) {
        match **s {
            ast::Symbol::Static { ref binding, ref value } => {
                self.ir.emit(ir::Symbol::Global {
                    name: *binding.name,
                    value: ir::Immediate(value.unwrap_literal().as_u32())
                });
            },
            ast::Symbol::Constant { .. } => {
                // Will be inlined on usage
            },
            ast::Symbol::Function { ref name, ref bindings, ref ret_ty, ref body } => {
                // Get the Binding out of the Node<Binding>
                let bindings: Vec<_> = bindings.iter().map(|b| **b).collect();

                self.trans_fn(**name, &bindings, *ret_ty, body);
            }
        }
    }
}


pub fn translate(ast: &[ast::Node<ast::Symbol>]) -> ir::Program {
    let mut visitor = Translator::new();
    walk_program(&mut visitor, ast);

    visitor.ir
}