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
//! than having to call a function to get a reference to the current block every
//! time we need to access it.

// TODO: SSA verifier?

use driver::interner::Ident;
use driver::session;
use front::ast;
use front::ast::visit::*;
use middle::ir::{self, Register};
use std::collections::{HashMap, HashSet, VecDeque};
use std::mem;

mod controlflow;
mod expr;

/// Information about the function that we're translating right now
///
/// ## Stack Slots and Registers
///
/// As RusTiny allows name shadowing in nested scopes, we need to
/// keep track of the alternative names we assign in case of
/// duplicates (`a` might be come `a1` or `a2` if shadowed). We
/// store these alternative names in `stack_slots` and `register`
/// respectively. Along with that we attach the alternative name
/// to symbol table. The symbol table not only stores variable
/// names but also keeps track of the scope the variable belongs
/// to.
///
/// This allows us to quickly check if a register name is already
/// in use by scanning `stack_slots` and `register` but we also
/// can retrieve the alternative name we assigned to a register
/// using the symbol table.
#[derive(Debug)]
struct FunctionContext {
    /// The generated function body
    body: Vec<ir::Block>,

    /// The function's arguments
    stack_slots: HashSet<Ident>,

    /// All registers used in this function
    registers: HashMap<Ident, Register>, // FIXME: Can we use a HashSet<Ident> instead here?

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

#[derive(Clone, Copy)]
pub enum VariableKind {
    Local,
    Stack,
    Static,
    Constant,
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
    Ignore,
}

/// The AST -> IR translator
pub struct Translator {
    ir: ir::Program,
    fcx: Option<FunctionContext>,
    /// As the translator might want to use the same label multiple times,
    /// we always append an index to it, which is stored here
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

        let id = Ident::from_str(&next_register.to_string());
        self.register_local(id);

        Register::Local(id)
    }

    /// Get the next free label with a given base name (e.g. `id` -> `id2`)
    fn next_free_label(&mut self, basename: Ident) -> ir::Label {
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
            Dest::Store(r) => {
                // For debugging: make sure the destination actually exists
                match r {
                    Register::Stack(id) => assert!(
                        self.fcx().stack_slots.contains(&id),
                        "Unregistered stack slot: {}",
                        id
                    ),
                    Register::Local(id) => assert!(
                        self.fcx().registers.contains_key(&id),
                        "Unregistered register: {}",
                        id
                    ),
                }

                r
            }
            Dest::Ignore => self.next_free_register(),
        }
    }

    fn with_first_block<F>(&mut self, current: &mut ir::Block, f: F)
    where
        F: Fn(&mut ir::Block) -> (),
    {
        match self.fcx().body.first_mut() {
            Some(first) => f(first),
            None => f(current),
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
            label,
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
        // Find a register name
        let mut id_mangled = id;
        let mut i = 1;

        while self.fcx()
            .registers
            .values()
            .any(|r| *r == Register::Local(id_mangled))
        {
            id_mangled = Ident::from_str(&format!("{}{}", id, i));
            i += 1;
        }

        // Register the variable
        let register = Register::Local(id_mangled);
        self.fcx().registers.insert(id, register);
        session()
            .symbol_table
            .set_register(self.fcx().scope, &id, register);

        register
    }

    /// Register a stack slot variable and return its register
    fn register_stack_slot(&mut self, id: Ident) -> Register {
        // Find a stack slot name
        let mut id_mangled = id;
        let mut i = 1;

        while self.fcx().stack_slots.contains(&id_mangled) {
            id_mangled = Ident::from_str(&format!("{}{}", id, i));
            i += 1;
        }

        // Register the variable
        self.fcx().stack_slots.insert(id_mangled);
        session()
            .symbol_table
            .set_slot(self.fcx().scope, &id, Register::Stack(id_mangled));

        Register::Stack(id_mangled)
    }

    /// Look up the alternative name we assigned in case of variable shadowing
    fn lookup_register(&mut self, reg: Register) -> Register {
        let var = session()
            .symbol_table
            .resolve_variable(self.fcx().scope, &reg.ident())
            .expect(&format!("variable {} not yet declared", reg));

        match reg {
            Register::Local(_) => var.reg.expect(&format!("No register assigned to {}", reg)),
            Register::Stack(_) => var.slot
                .expect(&format!("No stack slot assigned to {}", reg)),
        }
    }

    /// Determine the kind of variable the name refers to
    fn variable_kind(&mut self, name: &Ident) -> VariableKind {
        if self.fcx().registers.contains_key(name) {
            VariableKind::Local
        } else if self.fcx().stack_slots.contains(name) {
            VariableKind::Stack
        } else {
            match session().symbol_table.lookup_symbol(name) {
                Some(ast::Symbol::Static { .. }) => VariableKind::Static,
                Some(ast::Symbol::Constant { .. }) => VariableKind::Constant,
                _ => panic!("{} is not a variable", name),
            }
        }
    }

    /// Translate a function
    fn trans_fn(
        &mut self,
        name: Ident,
        bindings: &[ast::Binding],
        ret_ty: ast::Type,
        body: &ast::Node<ast::Block>,
    ) {
        let is_void = ret_ty == ast::Type::Unit;

        // Prepare function context
        self.fcx = Some(FunctionContext {
            body: Vec::new(),
            stack_slots: HashSet::new(),
            registers: HashMap::new(),
            return_slot: None,
            scope: body.id,
            next_register: 0,
            loop_exit: None,
        });

        // Prepare ast block
        let mut block = ir::Block {
            label: self.next_free_label(Ident::from_str("entry-block")),
            inst: VecDeque::new(),
            last: ir::ControlFlowInstruction::NotYetProcessed,
            phis: Vec::new(),
        };

        // Register arguments as stack slots
        // TODO: Maybe use calling convention here?
        // (only use stack for non-register arguments)
        for binding in bindings.iter().rev() {
            self.register_stack_slot(*binding.name);
        }

        // Translate ast block
        if is_void {
            self.trans_block(body, &mut block, Dest::Ignore);
        } else {
            // Store block value in temporary register...
            // NOTE: If the block contains a return expression, the ret_value register we pass
            // is skipped.
            let ret_slot = self.register_stack_slot(Ident::from_str("ret_slot"));
            self.fcx().return_slot = Some(ret_slot);

            let ret_value = self.next_free_register();
            self.trans_block(body, &mut block, Dest::Store(ret_value));
            // ... and store it in the memory slot
            if !block.finalized() {
                // If the block contains a 'return' expr, it already stores the result in the
                // return slot, so we don't have to do
                block.store(
                    ir::Value::Register(ret_value),
                    ir::Value::Register(ret_slot),
                );
            };
        };

        // Finalize the function
        if is_void && !block.finalized() {
            block.ret(None);
        } else {
            // Build the return block
            // FIXME: If there is a single store to the return slot,
            //        return it directly and skip the alloca/store
            let ret_slot = self.fcx().return_slot.unwrap();
            self.with_first_block(&mut block, |block| block.alloc(ret_slot));

            let return_label = self.next_free_label(Ident::from_str("return"));
            if !block.finalized() {
                block.jump(return_label);
            };

            // %reg = mem[%return_slot]
            // ret %reg
            self.commit_block_and_continue(&mut block, return_label);
            let return_value = self.next_free_register();
            block.load(ir::Value::Register(ret_slot), return_value);
            block.ret(Some(ir::Value::Register(return_value)));
        };

        self.commit_block(block);

        // Emit the symbol
        let fcx = self.fcx.take().unwrap();
        self.ir.emit(ir::Symbol::Function {
            name,
            body: fcx.body,
            args: bindings.iter().map(|b| *b.name).collect(),
        });
    }

    /// Translate an AST code block
    fn trans_block(&mut self, b: &ast::Node<ast::Block>, block: &mut ir::Block, dest: Dest) {
        with_reset!(self.fcx().scope, b.id, {
            for stmt in &b.stmts {
                self.trans_stmt(stmt, block);
            }

            self.trans_expr(&b.expr, block, dest);
        });
    }

    /// Translate a statement
    fn trans_stmt(&mut self, stmt: &ast::Statement, block: &mut ir::Block) {
        match *stmt {
            ast::Statement::Declaration {
                ref binding,
                ref value,
            } => {
                // Allocate memory on stack for the binding
                let dst = self.register_stack_slot(*binding.name);
                self.with_first_block(block, |block| block.alloc(dst));

                // Store the expression in the new slot
                let value = self.trans_expr_to_value(value, block);
                block.store_reg(value, dst);
            }
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
            ast::Symbol::Static {
                ref binding,
                ref value,
            } => {
                self.ir.emit(ir::Symbol::Global {
                    name: *binding.name,
                    value: ir::Immediate(value.unwrap_literal().as_u32()),
                });
            }
            ast::Symbol::Constant { .. } => {
                // Will be inlined on usage
            }
            ast::Symbol::Function {
                ref name,
                ref bindings,
                ref ret_ty,
                ref body,
            } => {
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
