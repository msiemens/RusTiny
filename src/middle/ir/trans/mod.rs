// TODO: SSA verifier?

use std::collections::{HashMap, LinkedList};
use std::mem;
use ::Ident;
use front::ast;
use middle::ir::{self, Register};
use front::ast::visit::*;


mod controlflow;
mod expr;


struct FunctionContext {
    body: Vec<ir::Block>,
    locals: HashMap<Ident, Register>,
    scope: ast::NodeId,
    return_slot: Option<Register>,

    next_id: u32,
    next_label: HashMap<Ident, u32>,

    loop_exit: Option<ir::Label>,
}


#[derive(Copy, Debug)]
enum Dest {
    Store(Register),
    Ignore
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

    fn unwrap_dest(&mut self, dest: Dest) -> Register {
        match dest {
            Dest::Store(r) => r,
            Dest::Ignore => self.next_free_register()
        }
    }

    fn commit_block_and_continue(&mut self, block: &mut ir::Block, label: ir::Label) {
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

    fn commit_block(&mut self, block: ir::Block) {
        self.fcx().body.push(block);
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
        let is_void = ret_ty == ast::Type::Unit;

        // Prepare function context
        let ret_slot = Register(Ident::new("ret_slot"));
        self.fcx = Some(FunctionContext {
            body: Vec::new(),
            locals: HashMap::new(),
            return_slot: if !is_void { Some(ret_slot) } else { None },
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
        for binding in bindings.iter().rev() {
            let reg = self.register_local(*binding.name);
            block.alloc(reg);
        }

        // Translate ast block
        if ret_ty != ast::Type::Unit {
            block.alloc(ret_slot);
            self.trans_block(body, &mut block, Dest::Store(ret_slot));
        } else {
            self.trans_block(body, &mut block, Dest::Ignore);
        }


        // Finish the function
        if is_void && !block.commited() {
            block.ret(None);
        } else {
            if !block.commited() {
                block.jump(ir::Label(Ident::new("return")));
            }

            // FIXME: If there is a single store to the return slot,
            //        return it directly and skip the alloca/store
            self.commit_block_and_continue(&mut block, ir::Label(Ident::new("return")));
            let return_value = self.next_free_register();
            block.load(ir::Value::Register(self.fcx().return_slot.unwrap()), return_value);
            block.ret(Some(ir::Value::Register(return_value)));
        }

        self.commit_block(block);

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
                   dest: Dest) {
        with_reset!(self.fcx().scope, b.id, {
            for stmt in &b.stmts {
                self.trans_stmt(stmt, block);
            }

            self.trans_expr(&**b.expr, block, dest);
        });
    }

    fn trans_stmt(&mut self,
                  stmt: &ast::Statement,
                  block: &mut ir::Block) {
        match *stmt {
            ast::Statement::Declaration { ref binding, ref value } => {
                // Allocate memory on stack for the binding
                let reg = self.register_local(*binding.name);
                match self.fcx().body.first_mut() {
                    Some(first) => first.alloc(reg),
                    None => block.alloc(reg)
                };

                // Store the expression in the new slot
                let value = self.trans_expr_to_value(value, block);
                block.store(value, reg);
            },
            ast::Statement::Expression { ref val } => {
                self.trans_expr(val, block, Dest::Ignore);
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
            ast::Symbol::Constant { .. } => {
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