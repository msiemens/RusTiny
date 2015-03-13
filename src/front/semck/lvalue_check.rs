//! Make sure the left-hand side of all assignments are variables
//!
//! For example, this program wouldn't compile:
//!
//! ```ignore
//! fn foo() {
//!     1 = false;
//! }
//! ```

use ast::*;
use util::visit::*;


struct LValueCheck;

impl LValueCheck {
    fn new() -> LValueCheck {
        LValueCheck
    }

    fn check_expr(&self, expr: &Node<Expression>) {
        if let Expression::Variable { .. } = **expr {
            return  // Everything's okay
        } else {
            fatal_at!("left-hand side of assignment is not a variable"; expr);
        }
    }
}

impl<'v> Visitor<'v> for LValueCheck {
    fn visit_expression(&mut self, expr: &'v Node<Expression>) {
        match **expr {
            Expression::Assign { ref lhs, .. } => self.check_expr(lhs),
            Expression::AssignOp { ref lhs, .. } => self.check_expr(lhs),
            _ => { walk_expression(self, expr) }
        }
    }
}

pub fn run(program: &Program) {
    let mut visitor = LValueCheck::new();
    walk_program(&mut visitor, program);
}