//! Make sure a `break` is always within a while loop

use front::ast::*;
use front::ast::visit::*;


struct BreakVerifier {
    while_depth: u32
}

impl BreakVerifier {
    fn new() -> BreakVerifier {
        BreakVerifier {
            while_depth: 0
        }
    }
}

impl<'v> Visitor<'v> for BreakVerifier {
    fn visit_expression(&mut self, expr: &'v Node<Expression>) {
        match **expr {
            Expression::While { .. } => {
                self.while_depth += 1;
            },
            Expression::Break => {
                if self.while_depth == 0 {
                    fatal_at!("`break` outside of loop"; expr);
                }
            },
            _ => {}
        }
    }
}

pub fn run(program: &[Node<Symbol>]) {
    let mut visitor = BreakVerifier::new();
    walk_program(&mut visitor, program);
}