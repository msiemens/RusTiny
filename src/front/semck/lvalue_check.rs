//! Make sure the left-hand side of all assignments are variables

use ast::*;
use driver::fatal_at;
use util::visit::*;


struct LValueCheck;

impl LValueCheck {
    fn new() -> LValueCheck {
        LValueCheck
    }

    fn check_expr(&self, expr: &Node<Expression>) {
        if let Expression::Variable { .. } = **expr {
            fatal_at(format!("left-hand side of assignment is not a variable"), expr);
        }
    }
}

impl<'v> Visitor<'v> for LValueCheck {
    fn visit_expression(&mut self, expr: &'v Node<Expression>) {
        match **expr {
            Expression::Assign { ref lhs, .. } => self.check_expr(lhs),
            Expression::AssignOp { ref lhs, .. } => self.check_expr(lhs),
            _ => {}
        }
    }
}

pub fn run(program: &mut Program) {
    let mut visitor = LValueCheck::new();
    walk_program(&mut visitor, program);
}