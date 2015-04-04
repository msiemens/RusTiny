use ::Ident;
use driver;
use driver::symbol_table::VariableKind;
use front::ast;
use middle::ir::{self, Register};
use middle::ir::trans::Translator;

impl Translator {
    pub fn trans_expr(&mut self,
                  expr: &ast::Expression,
                  block: &mut ir::Block,
                  dest: ir::Register) {
        match *expr {
            ast::Expression::Literal { ref val } => {
                self.trans_literal(val, block, dest);
            },
            ast::Expression::Variable { ref name } => {
                self.trans_variable(name, block, dest);
            },
            ast::Expression::Assign { ref lhs, ref rhs } => {
                self.trans_assign(lhs, rhs, block, dest)
            },
            ast::Expression::AssignOp { op, ref lhs, ref rhs } => {
                self.trans_assign_op(op, lhs, rhs, block, dest)
            },
            ast::Expression::Return { ref val } => {
                self.trans_return(val, block)
            },
            ast::Expression::Call { ref func, ref args } => {
                let args: Vec<_> = args.iter().map(|expr| &**expr).collect();
                self.trans_call(&func.unwrap_ident(), &args[..], block, dest);
            },
            ast::Expression::Group(ref expr) => {
                self.trans_expr(&**expr, block, dest);
            },
            ast::Expression::Infix { op, ref lhs, ref rhs } => {
                self.trans_infix(op, lhs, rhs, block, dest)
            },
            ast::Expression::Prefix { op, ref item } => {
                self.trans_prefix(op, item, block, dest)
            },
            ast::Expression::If { ref cond, ref conseq, ref altern } => {
                self.trans_if(cond, conseq, altern.as_ref().map(|b| &**b), block, dest)
            },
            ast::Expression::While { ref cond, ref body } => {
                self.trans_while(cond, body, block)
            },
            ast::Expression::Break => {
                self.trans_break(block)
            },
            ast::Expression::Unit => {}
        }
    }

    pub fn trans_expr_to_value(&mut self,
                               expr: &ast::Expression,
                               block: &mut ir::Block) -> ir::Value {
        // Special handling: ...
        if let ast::Expression::Literal { ref val } = *expr {
            return ir::Value::Immediate(ir::Immediate(val.as_u32()))
        }

        // Special handling: ...
        if let ast::Expression::Variable { ref name } = *expr {
            let sytable = &driver::session().symbol_table;
            let vkind = sytable.variable_kind(self.fcx().scope, name).unwrap();

            if let VariableKind::Constant = vkind {
                let symbol = sytable.lookup_symbol(name).unwrap();
                let val = symbol.get_value().unwrap_literal();
                return ir::Value::Immediate(ir::Immediate(val.as_u32()))
            }
        }

        let tmp = self.next_free_register();
        self.trans_expr(expr, block, tmp);
        ir::Value::Register(tmp)
    }

    fn trans_literal(&mut self,
                     val: &ast::Value,
                     block: &mut ir::Block,
                     dest: ir::Register) {
        let val = ir::Value::Immediate(ir::Immediate(val.as_u32()));
        block.store(val, dest)
    }

    fn trans_variable(&mut self,
                      name: &Ident,
                      block: &mut ir::Block,
                      dest: ir::Register) {
        let sytable = &driver::session().symbol_table;
        let vkind = sytable.variable_kind(self.fcx().scope, name).unwrap();

        match vkind {
            VariableKind::Local => {
                // %dest = load %local
                let r = self.fcx().locals[name];
                block.load(ir::Value::Register(r), dest);
            },
            VariableKind::Static => {
                // %dest = load %static
                block.load(ir::Value::Static(*name), dest);
            },
            VariableKind::Constant => {
                // %dest = {const}
                panic!("should already be handled!")
            },
        }
    }

    fn trans_assign(&mut self,
                    lhs: &ast::Expression,
                    rhs: &ast::Expression,
                    block: &mut ir::Block,
                    dest: ir::Register) {
        let reg = Register(lhs.unwrap_ident());  // Already registered
        let val = self.trans_expr_to_value(rhs, block);

        block.store(val, reg);
    }

    fn trans_assign_op(&mut self,
                       op: ast::BinOp,
                       lhs: &ast::Expression,
                       rhs: &ast::Expression,
                       block: &mut ir::Block,
                       dest: ir::Register) {
        let tmp = self.next_free_register();
        let dst = Register(lhs.unwrap_ident());  // Already registered

        self.trans_infix(op, lhs, rhs, block, tmp);
        block.store(ir::Value::Register(tmp), dst);
    }

    fn trans_call(&mut self,
                  func: &Ident,
                  args: &[&ast::Expression],
                  block: &mut ir::Block,
                  dest: ir::Register) {
        let translated_args: Vec<_> = args.iter().map(|expr| {
            self.trans_expr_to_value(expr, block)
        }).collect();

        block.call(*func, translated_args, dest);
    }

    fn trans_infix(&mut self,
                   op: ast::BinOp,
                   lhs: &ast::Expression,
                   rhs: &ast::Expression,
                   block: &mut ir::Block,
                   dest: ir::Register) {
        match op.get_type() {
            ast::BinOpType::Arithmetic | ast::BinOpType::Bitwise => {
            let lhs_val = self.trans_expr_to_value(lhs, block);
            let rhs_val = self.trans_expr_to_value(rhs, block);

                block.binop(ir::InfixOp::from_ast_op(op),
                            lhs_val,
                            rhs_val,
                            dest)
            },
            ast::BinOpType::Logic => {
                // Short-circuiting logic
                let label_lhs = block.label;
                let label_rhs = self.next_free_label(Ident::new("lazy-rhs"));
                let label_next = self.next_free_label(Ident::new("lazy-next"));

                let lhs_val = self.trans_expr_to_value(lhs, block);

                match op {
                    ast::BinOp::And => block.branch(lhs_val, label_rhs, label_next),
                    ast::BinOp::Or  => block.branch(lhs_val, label_next, label_rhs),
                    _ => panic!()
                }

                self.commit_pending_block(block, label_rhs);
                let rhs_val = self.trans_expr_to_value(rhs, block);
                block.jump(label_next);

                self.commit_pending_block(block, label_next);
                block.phi(vec![(lhs_val, label_lhs),
                               (rhs_val, label_rhs)],
                          dest);
            },
            ast::BinOpType::Comparison => {
                let lhs_val = self.trans_expr_to_value(lhs, block);
                let rhs_val = self.trans_expr_to_value(rhs, block);

                block.cmp(ir::CmpOp::from_ast_op(op),
                          lhs_val,
                          rhs_val,
                          dest)
            }
        }
    }

    fn trans_prefix(&mut self,
                   op: ast::UnOp,
                   item: &ast::Expression,
                   block: &mut ir::Block,
                   dest: ir::Register) {
        let tmp = self.trans_expr_to_value(item, block);

        block.unop(ir::PrefixOp::from_ast_op(op),
                    tmp,
                    dest)
    }
}