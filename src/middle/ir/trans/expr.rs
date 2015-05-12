//! Translate of expressions

use ::Ident;
use driver;
use driver::symbol_table::VariableKind;
use front::ast;
use middle::ir::{self, Register};
use middle::ir::trans::{Dest, Translator};

impl Translator {
    /// Translate an expression
    pub fn trans_expr(&mut self,
                  expr: &ast::Expression,
                  block: &mut ir::Block,
                  dest: Dest) {
        match *expr {
            ast::Expression::Literal { ref val } => {
                self.trans_literal(val, block, dest);
            },
            ast::Expression::Variable { ref name } => {
                self.trans_variable(name, block, dest);
            },
            ast::Expression::Assign { ref lhs, ref rhs } => {
                self.trans_assign(lhs, rhs, block)
            },
            ast::Expression::AssignOp { op, ref lhs, ref rhs } => {
                self.trans_assign_op(op, lhs, rhs, block)
            },
            ast::Expression::Return { ref val } => {
                self.trans_return(val, block)
            },
            ast::Expression::Call { ref func, ref args } => {
                // Get the &Expr out of the &Node<Expr>
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

    /// Translate an expression into an ir::Value
    ///
    /// Often a function might take a register or an immediate value
    /// (think of arithmetics). We want literal values and constants to evaluate
    /// to immediate values but every other expression to a register that contains
    /// the result. Enter `trans_expr_to_value`:
    pub fn trans_expr_to_value(&mut self,
                               expr: &ast::Expression,
                               block: &mut ir::Block) -> ir::Value {
        // Special handling for literals: return the immediate value
        if let ast::Expression::Literal { ref val } = *expr {
            return ir::Value::Immediate(ir::Immediate(val.as_u32()))
        }

        if let ast::Expression::Variable { ref name } = *expr {
            // Look up of which kind the variable is
            let sytable = &driver::session().symbol_table;
            let vkind = sytable.variable_kind(self.fcx().scope, name).unwrap();

            // Special handling for constants: return the immediate value
            if let VariableKind::Constant = vkind {
                // Get the constant's value and return it as an immediate
                let symbol = sytable.lookup_symbol(name).unwrap();
                let val = symbol.get_value().unwrap_literal();
                return ir::Value::Immediate(ir::Immediate(val.as_u32()))
            }
        }

        // Evaluate into a temporary register and return it
        let tmp = self.next_free_register();
        self.trans_expr(expr, block, Dest::Store(tmp));
        ir::Value::Register(tmp)
    }

    fn assign_dest(&mut self, dest: Ident) -> ir::Value {
        // Look up of which kind the variable is
        let sytable = &driver::session().symbol_table;
        let vkind = sytable.variable_kind(self.fcx().scope, &dest).unwrap();

        match vkind {
            VariableKind::Local => ir::Value::Register(self.lookup_register(&dest)),
            VariableKind::Static => ir::Value::Static(dest),
            VariableKind::Constant => panic!("attempt to assign to a constant"),
        }
    }

    /// Translate a literal
    fn trans_literal(&mut self,
                     val: &ast::Value,
                     block: &mut ir::Block,
                     dest: Dest) {
        let val = ir::Value::Immediate(ir::Immediate(val.as_u32()));
        let dst = self.unwrap_dest(dest);
        block.store_reg(val, dst)
    }

    /// Translate the usage of a variable
    fn trans_variable(&mut self,
                      name: &Ident,
                      block: &mut ir::Block,
                      dest: Dest) {
        let sytable = &driver::session().symbol_table;
        let vkind = sytable.variable_kind(self.fcx().scope, name).unwrap();

        match vkind {
            VariableKind::Local => {
                // %dest = load %local
                let reg = self.lookup_register(name);
                block.load(ir::Value::Register(reg), self.unwrap_dest(dest));
            },
            VariableKind::Static => {
                // %dest = load %static
                block.load(ir::Value::Static(*name), self.unwrap_dest(dest));
            },
            VariableKind::Constant => {
                // %dest = {const}
                panic!("should already be handled!")
            },
        }
    }

    /// Translate an assignment
    fn trans_assign(&mut self,
                    lhs: &ast::Expression,
                    rhs: &ast::Expression,
                    block: &mut ir::Block) {
        let dst = self.assign_dest(lhs.unwrap_ident());
        let val = self.trans_expr_to_value(rhs, block);

        block.store(val, dst);
    }

    /// Translate an assignment with an operator
    fn trans_assign_op(&mut self,
                       op: ast::BinOp,
                       lhs: &ast::Expression,
                       rhs: &ast::Expression,
                       block: &mut ir::Block) {
        let dst = self.assign_dest(lhs.unwrap_ident());
        let tmp = self.next_free_register();

        self.trans_infix(op, lhs, rhs, block, Dest::Store(tmp));
        block.store(ir::Value::Register(tmp), dst);
    }

    /// Translate a function call
    fn trans_call(&mut self,
                  func: &Ident,
                  args: &[&ast::Expression],
                  block: &mut ir::Block,
                  dest: Dest) {
        let translated_args: Vec<_> = args.iter().map(|expr| {
            self.trans_expr_to_value(expr, block)
        }).collect();

        block.call(*func, translated_args, self.unwrap_dest(dest));
    }

    /// Translate an infix expression
    fn trans_infix(&mut self,
                   op: ast::BinOp,
                   lhs: &ast::Expression,
                   rhs: &ast::Expression,
                   block: &mut ir::Block,
                   dest: Dest) {
        // FIXME: Docs
        match op.get_type() {
            ast::BinOpType::Arithmetic | ast::BinOpType::Bitwise => {
                let lhs_val = self.trans_expr_to_value(lhs, block);
                let rhs_val = self.trans_expr_to_value(rhs, block);

                block.binop(ir::InfixOp::from_ast_op(op),
                            lhs_val,
                            rhs_val,
                            self.unwrap_dest(dest))
            },
            ast::BinOpType::Logic => {
                // Short-circuiting logic. This involves branching to skip the
                // right-hand side part if possible. FIXME: more explanation
                let label_lhs = block.label;
                let label_rhs = self.next_free_label(Ident::new("lazy-rhs"));
                let label_next = self.next_free_label(Ident::new("lazy-next"));

                // The left-hand side
                let lhs_val = self.trans_expr_to_value(lhs, block);

                // FIXME: Explanation
                match op {
                    ast::BinOp::And => block.branch(lhs_val, label_rhs, label_next),
                    ast::BinOp::Or  => block.branch(lhs_val, label_next, label_rhs),
                    _ => panic!()
                }

                // Evaluate the right-hand side
                self.commit_block_and_continue(block, label_rhs);
                let rhs_val = self.trans_expr_to_value(rhs, block);
                block.jump(label_next);

                // Select the value (lhs vs rhs) based on where we came from
                // (by using the Phi function).
                self.commit_block_and_continue(block, label_next);
                block.phi(vec![(lhs_val, label_lhs),
                               (rhs_val, label_rhs)],
                          self.unwrap_dest(dest));
            },
            ast::BinOpType::Comparison => {
                let lhs_val = self.trans_expr_to_value(lhs, block);
                let rhs_val = self.trans_expr_to_value(rhs, block);

                block.cmp(ir::CmpOp::from_ast_op(op),
                          lhs_val,
                          rhs_val,
                          self.unwrap_dest(dest))
            }
        }
    }

    /// Translate a prefix operation
    fn trans_prefix(&mut self,
                   op: ast::UnOp,
                   item: &ast::Expression,
                   block: &mut ir::Block,
                   dest: Dest) {
        let tmp = self.trans_expr_to_value(item, block);

        block.unop(ir::PrefixOp::from_ast_op(op),
                    tmp,
                    self.unwrap_dest(dest))
    }
}