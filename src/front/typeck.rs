//! Type checking
//!
//! We need to make sure the program uses types coherently. To do that,
//! we walk the AST and collect type information for every node. While
//! doing that we check some coherence rules (e.g. addition requires two ints).
//! If types mismatch, an error is reported.

use driver::session;
use driver::symbol_table::SymbolTable;
use front::ast::visit::*;
use front::ast::*;
use std::collections::HashMap;

/// Information about the current function
struct FunctionContext {
    /// The function's return type
    return_ty: Type,
    /// Whether the body has an explicit return statement
    explicit_return: bool,
}

struct TypeCheck<'a> {
    sytbl: &'a SymbolTable,
    types: HashMap<NodeId, Type>,
    scope: NodeId,
    fctx: FunctionContext,
}

impl<'a> TypeCheck<'a> {
    fn new(sytbl: &'a SymbolTable) -> TypeCheck<'a> {
        TypeCheck {
            sytbl,
            types: HashMap::new(),
            scope: NodeId(!0),
            fctx: FunctionContext {
                return_ty: Type::Unit,
                explicit_return: false,
            },
        }
    }

    fn type_check<T>(&self, ty: Type, expected: Type, node: &Node<T>) {
        if ty == Type::Err {
            // Assume there's nothing wrong to collect more type errors
            return;
        }

        if ty != expected {
            fatal_at!("type mismatch: expected {}, got {}", expected, ty; node);
        }
    }

    fn check_fn(&mut self, return_ty: Type, body: &Node<Block>) {
        self.fctx = FunctionContext {
            return_ty,
            explicit_return: false,
        };

        let implicit_ret_ty = self.check_block(body, None);

        if self.fctx.explicit_return {
            // There was an explicit return. Thus, the body has to evaluate
            // to ().
            self.type_check(implicit_ret_ty, Type::Unit, body);
        } else if implicit_ret_ty == Type::Unit && return_ty != Type::Unit {
            // There was NO explicit return. Thus, the body has to evaluate
            // to the return type from the function signature.
            fatal_at!("missing return value/return statement"; body);
        } else {
            self.type_check(implicit_ret_ty, return_ty, body);
        }
    }

    fn check_block(&mut self, block: &Node<Block>, expected_ty: Option<Type>) -> Type {
        with_reset!(self.scope, block.id, {
            for stmt in &block.stmts {
                self.check_statement(stmt);
            }

            let block_ty = self.check_expression(&block.expr, expected_ty);
            self.types.insert(block.id, block_ty);

            block_ty
        })
    }

    fn check_statement(&mut self, stmt: &Node<Statement>) {
        match **stmt {
            Statement::Declaration {
                ref binding,
                ref value,
            } => {
                let var = self
                    .sytbl
                    .resolve_variable(self.scope, &binding.name)
                    .unwrap();
                self.check_expression(value, Some(var.ty));
            }
            Statement::Expression { ref val } => {
                // Expression can be of any type as the statement always
                // evaluates to ()
                self.check_expression(val, None);
            }
        }
    }

    /// Typecheck an expression
    ///
    /// If `expected` is None, there is no specific expectation from the parent.
    /// Otherwise an error will be repored if the types do not match.
    ///
    /// This is the real meat of type checking.
    fn check_expression(&mut self, expr: &Node<Expression>, expected: Option<Type>) -> Type {
        let ty = match **expr {
            // Basic expressions:
            Expression::Literal { ref val } => val.get_ty(),
            Expression::Variable { ref name } => {
                let scope = self.scope;
                self.sytbl
                    .resolve_variable(scope, name)
                    .unwrap_or_else(|| panic!("no variable named {}", name))
                    .ty
            }
            // Compound expressions
            Expression::Assign { ref lhs, ref rhs } => self.check_assign(lhs, rhs),
            Expression::AssignOp {
                ref op,
                ref lhs,
                ref rhs,
            } => self.check_assign_op(op, lhs, rhs),
            Expression::Return { ref val } => self.check_return(val),
            Expression::Call { ref func, ref args } => self.check_call(func, args),
            Expression::Group(ref expr) => self.check_expression(expr, expected),
            Expression::Infix {
                ref op,
                ref lhs,
                ref rhs,
            } => self.check_infix(op, lhs, rhs),
            Expression::Prefix { ref op, ref item } => self.check_prefix(op, item),
            Expression::If {
                ref cond,
                ref conseq,
                ref altern,
            } => self.check_if(cond, conseq, altern, expected),
            Expression::While { ref cond, ref body } => self.check_while(cond, body),
            Expression::Break | Expression::Unit => Type::Unit,
        };

        // Store result in type cache
        self.types.insert(expr.id, ty);

        if let Some(expected) = expected {
            self.type_check(ty, expected, expr)
        }

        ty
    }

    fn check_assign(&mut self, lhs: &Node<Expression>, rhs: &Node<Expression>) -> Type {
        // Infer from left hand side
        let expected = self.check_expression(lhs, None);
        self.check_expression(rhs, Some(expected));

        Type::Unit
    }

    fn check_assign_op(
        &mut self,
        op: &BinOp,
        lhs: &Node<Expression>,
        rhs: &Node<Expression>,
    ) -> Type {
        self.check_infix(op, lhs, rhs);
        Type::Unit
    }

    fn check_return(&mut self, val: &Node<Expression>) -> Type {
        self.fctx.explicit_return = true;

        let expected = self.fctx.return_ty;
        self.check_expression(val, Some(expected))
    }

    fn check_call(&mut self, func: &Node<Expression>, args: &[Node<Expression>]) -> Type {
        let (bindings, ret_ty) = self.sytbl.lookup_function(&func.unwrap_ident()).unwrap();

        // Check argument count
        if args.len() != bindings.len() {
            fatal_at!("mismatching argument count: expected {}, got {}", bindings.len(), args.len(); func);
            return Type::Err;
        }

        // Check argument types
        for (arg, binding) in args.iter().zip(bindings.into_iter()) {
            self.check_expression(arg, Some(binding.ty));
        }

        ret_ty
    }

    fn check_infix(&mut self, op: &BinOp, lhs: &Node<Expression>, rhs: &Node<Expression>) -> Type {
        match op.get_type() {
            BinOpType::Arithmetic => {
                self.check_expression(lhs, Some(Type::Int));
                self.check_expression(rhs, Some(Type::Int));
                Type::Int
            }
            BinOpType::Logic => {
                self.check_expression(lhs, Some(Type::Bool));
                self.check_expression(rhs, Some(Type::Bool));
                Type::Bool
            }
            BinOpType::Bitwise => {
                // Both Ints and Bools are accepted here, thus we infer the
                // used type from the left hand side
                let ty = self.check_expression(lhs, None);
                if ty == Type::Bool || ty == Type::Int {
                    self.check_expression(rhs, Some(ty));
                    ty
                } else {
                    fatal_at!("binary operation `{}` cannot be applied to {}", op, ty; lhs);
                    Type::Err
                }
            }
            BinOpType::Comparison => {
                self.check_expression(lhs, Some(Type::Int));
                self.check_expression(rhs, Some(Type::Int));
                Type::Bool
            }
        }
    }

    fn check_prefix(&mut self, op: &UnOp, item: &Node<Expression>) -> Type {
        match *op {
            UnOp::Neg => {
                self.check_expression(item, Some(Type::Int));
                Type::Int
            }
            UnOp::Not => {
                let ty = self.check_expression(item, None);
                if ty == Type::Bool || ty == Type::Int {
                    ty
                } else {
                    fatal_at!("unary operation `{}` cannot be applied to {}", op, ty; item);
                    Type::Err
                }
            }
        }
    }

    fn check_if(
        &mut self,
        cond: &Node<Expression>,
        conseq: &Node<Block>,
        altern: &Option<Box<Node<Block>>>,
        expected: Option<Type>,
    ) -> Type {
        self.check_expression(cond, Some(Type::Bool));

        // Verify that the conseq type is matches `expected` ...
        // ... or infer it if `expected` is None
        let conseq_ty = self.check_block(conseq, expected);

        if let Some(ref altern) = *altern {
            self.check_block(altern, Some(conseq_ty));
        } else if let Some(expected) = expected {
            if expected != Type::Unit {
                fatal_at!("missing else clause"; conseq);
            }
        }

        if let Some(expected) = expected {
            // The containing expr/statement would usually check the conseq type
            // again. But if there was a type error, it will have been reported
            // by the check_block(conseq, expected) above. Thus, we return a type
            // error to not throw multiple errors.
            if expected != conseq_ty {
                return Type::Err;
            }
        }

        conseq_ty
    }

    fn check_while(&mut self, cond: &Node<Expression>, body: &Node<Block>) -> Type {
        self.check_expression(cond, Some(Type::Bool));
        self.check_block(body, Some(Type::Unit));
        Type::Unit
    }
}

impl<'v> Visitor<'v> for TypeCheck<'v> {
    fn visit_symbol(&mut self, symbol: &'v Node<Symbol>) {
        match **symbol {
            Symbol::Function {
                ref ret_ty,
                ref body,
                ..
            } => {
                self.check_fn(*ret_ty, body);
            }
            Symbol::Static {
                ref binding,
                ref value,
            }
            | Symbol::Constant {
                ref binding,
                ref value,
            } => {
                self.check_expression(value, Some(binding.ty));
            }
        }
    }
}

pub fn run(program: &[Node<Symbol>]) {
    let symbol_table = &session().symbol_table;
    let mut visitor = TypeCheck::new(symbol_table);
    walk_program(&mut visitor, program);

    session().abort_if_errors();
}
