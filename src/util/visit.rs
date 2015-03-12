//! An AST walker
//!
//! Allows it to walk different parts of the AST without rewriting the whole
//! logic over and over again. Uses the Visitor pattern. To continue walking
//! the AST, the corresponding `walk_*` method should be called at the end
//! of `visit_*`. Walks all nodes in-order (implementations are allowed to
//! rely on it!).

use ast::*;


pub trait Visitor<'v> : Sized {
    fn visit_symbol(&mut self, s: &'v Node<Symbol>) {
        walk_symbol(self, s)
    }

    fn visit_binding(&mut self, b: &'v Node<Binding>) {
        walk_binding(self, b)
    }

    fn visit_ident(&mut self, _: &'v Node<Ident>) {
        // Nothing to do
    }

    fn visit_type(&mut self, _: &'v Type) {
        // Nothing to do
    }

    fn visit_block(&mut self, b: &'v Node<Block>) {
        walk_block(self, b)
    }

    fn visit_statement(&mut self, stmt: &'v Node<Statement>) {
        walk_statement(self, stmt)
    }

    fn visit_expression(&mut self, e: &'v Node<Expression>) {
        walk_expression(self, e)
    }
}


pub fn walk_program<'v, V>(visitor: &mut V, program: &'v Program)
        where V: Visitor<'v>
{
    for symbol in program {
        visitor.visit_symbol(&*symbol)
    }
}

pub fn walk_symbol<'v, V>(visitor: &mut V, symbol: &'v Node<Symbol>)
        where V: Visitor<'v>
{
    match **symbol {
        Symbol::Static { ref binding, .. } => {
            visitor.visit_binding(&*binding);
            //visitor.visit_value(&*value);
        },
        Symbol::Constant { ref binding, .. } => {
            visitor.visit_binding(&*binding);
            //visitor.visit_value(&*value);
        },
        Symbol::Function { ref name, ref bindings, ref ret_ty, ref body } => {
            visitor.visit_ident(&*name);
            for binding in bindings {
                visitor.visit_binding(&*binding);
            }
            visitor.visit_type(&*ret_ty);
            visitor.visit_block(&*body);
        }
    }
}

pub fn walk_binding<'v, V>(visitor: &mut V, binding: &'v Node<Binding>)
        where V: Visitor<'v>
{
    visitor.visit_type(&*&binding.ty);
    visitor.visit_ident(&*&binding.name);
}


pub fn walk_block<'v, V>(visitor: &mut V, block: &'v Node<Block>)
        where V: Visitor<'v>
{
    for stmt in &block.stmts {
        visitor.visit_statement(&*stmt);
    }

    if let Some(ref expr) = block.expr {
        visitor.visit_expression(&*expr);
    }
}


pub fn walk_statement<'v, V>(visitor: &mut V, stmt: &'v Node<Statement>)
        where V: Visitor<'v>
{
    match **stmt {
        Statement::Declaration { ref binding, .. } => visitor.visit_binding(&*binding),
        Statement::Expression { ref val } => visitor.visit_expression(&*val)
    }
}


pub fn walk_expression<'v, V>(visitor: &mut V, expr: &'v Node<Expression>)
        where V: Visitor<'v>
{
    match **expr {
        Expression::Literal { .. } => {},
        Expression::Variable { ref name } => {
            visitor.visit_ident(&*name)
        },
        Expression::Assign { ref lhs, ref rhs } => {
            visitor.visit_expression(&*lhs);
            visitor.visit_expression(&*rhs);
        },
        Expression::AssignOp { op: _, ref lhs, ref rhs } => {
            visitor.visit_expression(&*lhs);
            visitor.visit_expression(&*rhs);
        },
        Expression::Return { ref val } => {
            if let Some(ref val) = *val {
                visitor.visit_expression(&*val);
            }
        },
        Expression::Call { ref func, ref args } => {
            visitor.visit_expression(&*func);
            for arg in args {
                visitor.visit_expression(&*arg);
            }
        },
        Expression::Group(ref expr) => visitor.visit_expression(&*expr),
        Expression::Infix { op: _, ref lhs, ref rhs } => {
            visitor.visit_expression(&*lhs);
            visitor.visit_expression(&*rhs);
        },
        Expression::Prefix { op: _, ref item } => {
            visitor.visit_expression(&*item);
        },
        Expression::If { ref cond, ref conseq, ref altern } => {
            visitor.visit_expression(&*cond);
            visitor.visit_block(&*conseq);
            if let Some(ref else_block) = *altern {
                visitor.visit_block(&*else_block);
            }
        },
        Expression::While { ref cond, ref body } => {
            visitor.visit_expression(&*cond);
            visitor.visit_block(&*body);
        },
        Expression::Break => {}
    }
}