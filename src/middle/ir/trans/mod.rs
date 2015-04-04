// TODO: (done) Check that expr -> Variable comments are right
// TODO: (done) Generate allocas for arguments
// TODO: (done) Fix test.rs IR
// TODO: (done) Pretty-print function arguments
// TODO: (done) Translate statement expressions
// TODO: (done) Fix test failures
// TODO: (done) Translate logical operators

// TODO: SSA verifier?
// TODO: Cleanup pass: remove `jmp label; label: ...`

use std::collections::{HashMap, LinkedList};
use std::mem;
use front::ast::{self, Ident};
use middle::ir::{self, Register};
use front::ast::visit::*;


mod controlflow;
mod expr;


struct FunctionContext {
    body: Vec<ir::Block>,
    locals: HashMap<Ident, Register>,
    scope: ast::NodeId,
    return_slot: Register,

    next_id: u32,
    next_label: HashMap<Ident, u32>,

    loop_exit: Option<ir::Label>,
}


pub struct Translator {
    ir: ir::Program,
    fcx: Option<FunctionContext>,
}

impl Translator {
    pub fn new() -> Translator {
        Translator {
            ir: Vec::new(),
            fcx: None
        }
    }

    fn fcx(&mut self) -> &mut FunctionContext {
        self.fcx.as_mut().unwrap()
    }

    fn next_free_register(&mut self) -> Register {
        let next_id = self.fcx().next_id;
        self.fcx().next_id += 1;

        let register = self.register_local(Ident::new(&next_id.to_string()));

        register
    }

    fn next_free_label(&mut self, basename: Ident) -> ir::Label {
        let ref mut next_label = self.fcx().next_label;

        *next_label.entry(basename).or_insert(0) += 1;

        let mut name = basename.to_string();
        name.push_str(&next_label[&basename].to_string());
        ir::Label(Ident::new(&name))
    }

    fn commit_pending_block(&mut self, block: &mut ir::Block, label: ir::Label) {
        assert!(block.last != ir::ControlFlowInstruction::NotYetProcessed);

        let mut new_block = ir::Block {
            label: label,
            inst: LinkedList::new(),
            last: ir::ControlFlowInstruction::NotYetProcessed
        };

        mem::swap(block, &mut new_block);

        // new_block is now the old block
        self.fcx().body.push(new_block);
    }

    fn register_local(&mut self, id: Ident) -> Register {
        let mut id_mangled = id;
        let mut i = 1;

        // Find a free name
        while self.fcx().locals.contains_key(&id_mangled) {
            id_mangled = Ident::new(&format!("{}{}", id, i));
            i += 1;
        }

        let register = Register(id_mangled);
        assert!(self.fcx().locals.insert(id_mangled, register).is_none());

        register
    }

    fn trans_fn(&mut self,
                name: Ident,
                bindings: &[ast::Binding],
                ret_ty: ast::Type,
                body: &ast::Node<ast::Block>) {
        // Prepare function context
        let ret_slot = Register(Ident::new("ret_slot"));
        self.fcx = Some(FunctionContext {
            body: Vec::new(),
            locals: HashMap::new(),
            return_slot: ret_slot,
            scope: ast::NodeId(-1),
            next_id: 0,
            next_label: HashMap::new(),
            loop_exit: None
        });

        // Prepare ast block
        let mut block = ir::Block {
            label: ir::Label(Ident::new("entry-block")),
            inst: LinkedList::new(),
            last: ir::ControlFlowInstruction::NotYetProcessed
        };

        // Allocate arguments & return slot
        for binding in bindings {
            let reg = self.register_local(*binding.name);
            block.alloc(reg);
        }

        // FIXME: If not void
        if ret_ty != ast::Type::Unit {
            block.alloc(ret_slot);
        }

        // Translate ast block
        self.trans_block(body, &mut block, ret_slot);

        // Finish the function
        // FIXME: If there is a single store to the return slot,
        //        return it directly and skip the alloca/store
        if !block.commited() {
            block.jump(ir::Label(Ident::new("return")));
        }

        self.commit_pending_block(&mut block, ir::Label(Ident::new("return")));

        match ret_ty {
            ast::Type::Unit => {
                block.ret(None);
            },
            _ => {
                let return_value = self.next_free_register();
                block.load(ir::Value::Register(self.fcx().return_slot), return_value);
                block.ret(Some(ir::Value::Register(return_value)));
            }
        }

        self.fcx().body.push(block);

        // Emit the symbol
        let fcx = self.fcx.take().unwrap();
        self.emit_symbol(ir::Symbol::Function {
            name: name,
            body: fcx.body,
            args: bindings.iter().map(|b| *b.name).collect(),
            locals: fcx.locals,
        });
    }

    fn trans_block(&mut self,
                   b: &ast::Node<ast::Block>,
                   block: &mut ir::Block,
                   dest: ir::Register) {
        let old_scope = self.fcx().scope;
        self.fcx().scope = b.id;

        for stmt in &b.stmts {
            self.trans_stmt(stmt, block);
        }

        self.trans_expr(&**b.expr, block, dest);

        self.fcx().scope = old_scope;
    }

    fn trans_stmt(&mut self,
                  stmt: &ast::Statement,
                  block: &mut ir::Block) {
        match *stmt {
            ast::Statement::Declaration { ref binding, ref value } => {
                // Allocate memory on stack for the binding
                let slot = block.alloc(self.register_local(*binding.name));

                // Store the expression in the new slot
                let value = self.trans_expr_to_value(value, block);
                block.store(value, slot);

                // FIXME: Insert in first block!
            },
            ast::Statement::Expression { ref val } => {
                // FIXME: Don't increment register for while/return/break
                let r = self.next_free_register();
                self.trans_expr(val, block, r);
            }
        }
    }


    fn emit_symbol(&mut self, s: ir::Symbol) {
        self.ir.push(s);
    }
}


impl<'v> Visitor<'v> for Translator {
    fn visit_symbol(&mut self, s: &'v ast::Node<ast::Symbol>) {
        match **s {
            ast::Symbol::Static { ref binding, ref value } => {
                self.emit_symbol(ir::Symbol::Global {
                    name: *binding.name,
                    value: ir::Immediate(value.unwrap_literal().as_u32())
                });
            },
            ast::Symbol::Constant { ref binding, ref value } => {
                // Will be inlined on usage
            },
            ast::Symbol::Function { ref name, ref bindings, ref ret_ty, ref body } => {
                let bindings: Vec<_> = bindings.iter().map(|b| (**b).clone()).collect();

                self.trans_fn(**name,
                              &*bindings,
                              *ret_ty,
                              body);
            }
        }
    }
}


pub fn translate(ast: &ast::Program) -> ir::Program {
    let mut visitor = Translator::new();
    walk_program(&mut visitor, ast);

    visitor.ir
}