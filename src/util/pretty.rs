//! A pretty printer
//!
//! Prints a parsed AST so that parsing it again would result in the same AST
//! again.

use std::io::Write;
use ast::*;


pub struct PrettyPrinter<'a, W: 'a> {
    indent: u32,
    program: &'a Program,
    out: &'a mut W
}

impl<'a, W: Write> PrettyPrinter<'a, W> {
    pub fn print(program: &'a Program, out: &mut W) {
        PrettyPrinter {
            indent: 0,
            program: program,
            out: out
        }.print_program();
    }

    fn print_indent(&mut self) {
        for _ in 0..(self.indent * 4) {
            write!(&mut self.out, " ").ok();
        }
    }

    fn print_program(&mut self) {
        for symbol in self.program {
            self.print_symbol(&symbol);
        }
    }

    fn print_symbol(&mut self, symbol: &Symbol) {
        match *symbol {
            Symbol::Static   { ref binding, ref value } => self.print_static(&binding, value),
            Symbol::Constant { ref binding, ref value } => self.print_constant(&binding, value),
            Symbol::Function {
                ref name,
                ref bindings,
                ref ret_ty,
                ref body
            } => self.print_function(&name, &bindings, ret_ty, &body)
        }
    }

    fn print_static(&mut self, binding: &Binding, value: &Value) {
        write!(self.out, "static {:?} = {:?};\n", binding, value).ok();
    }

    fn print_constant(&mut self, binding: &Binding, value: &Value) {
        write!(self.out, "const {:?} = {:?};\n", binding, value).ok();
    }

    fn print_function(&mut self,
                      name: &str,
                      bindings: &[Node<Binding>],
                      ret_ty: &Type,
                      body: &Block)
    {
        write!(self.out, "\n").ok();
        write!(&mut self.out, "fn {}({}) -> {:?} ",
               name,
               bindings
                  .iter()
                  .map(|b| format!("{:?}", b))
                  .collect::<Vec<_>>()
                  .connect(", "),
               ret_ty).ok();
        self.print_block(body)
    }

    fn print_block(&mut self, block: &Block) {
        //self.print_indent();
        write!(self.out, "{{\n").ok();

        self.indent += 1;

        for stmt in &block.stmts {
            self.print_statement(&stmt);
        }

        if let Expression::Unit = **block.expr {}
        else {
            self.print_indent();
            self.print_expression(&**block.expr);
            write!(self.out, "\n").ok();
        }

        self.indent -= 1;

        self.print_indent();
        write!(&mut self.out, "}}").ok();
    }

    fn print_statement(&mut self, stmt: &Statement) {
        self.print_indent();

        match *stmt {
            Statement::Declaration { ref binding, ref value } => {
                write!(&mut self.out, "let {:?} = ", binding).ok();
                self.print_expression(value);
                write!(self.out, ";\n").ok();
            },
            Statement::Expression { ref val } => {
                self.print_expression(val);
                write!(self.out, ";\n").ok();
            }
        }
    }

    fn print_expression(&mut self, expr: &Expression) {
        match *expr {
            Expression::Group(ref expr) => {
                write!(&mut self.out, "(").ok();
                self.print_expression(expr);
                write!(&mut self.out, ")").ok();
            },
            Expression::Call { ref func, ref args } => {
                self.print_expression(func);
                write!(&mut self.out, "(").ok();
                for arg in args {
                    self.print_expression(arg);
                    write!(&mut self.out, ", ").ok();
                }
                write!(&mut self.out, ")").ok();
            },
            Expression::Infix { ref op, ref lhs, ref rhs } => {
                self.print_expression(lhs);
                write!(&mut self.out, " {:?} ", op).ok();
                self.print_expression(rhs);
            },
            Expression::Prefix { ref op, ref item } => {
                write!(&mut self.out, "{:?}", op).ok();
                self.print_expression(item);
            },
            Expression::Literal { ref val } => {
                write!(&mut self.out, "{:?}", val).ok();
            },
            Expression::Variable { ref name } => {
                write!(&mut self.out, "{:?}", name).ok();
            },
            Expression::If { ref cond, ref conseq, ref altern } => {
                write!(&mut self.out, "if ").ok();
                self.print_expression(cond);
                write!(&mut self.out, " ").ok();
                self.print_block(conseq);
                match *altern {
                    Some(ref b) => {
                        write!(&mut self.out, " else ").ok();
                        self.print_block(b);
                    },
                    None => {}
                }
            },
            Expression::While { ref cond, ref body } => {
                write!(&mut self.out, "while ").ok();
                self.print_expression(cond);
                write!(&mut self.out, " ").ok();
                self.print_block(body);
            },
            Expression::Assign { ref lhs, ref rhs } => {
                self.print_expression(lhs);
                write!(&mut self.out, " = ").ok();
                self.print_expression(rhs);
            },
            Expression::AssignOp { ref op, ref lhs, ref rhs } => {
                self.print_expression(lhs);
                write!(&mut self.out, " {:?}= ", op).ok();
                self.print_expression(rhs);
            },
            Expression::Break => {
                write!(&mut self.out, "break").ok();
            },
            Expression::Return { ref val } => {
                write!(&mut self.out, "return ").ok();
                self.print_expression(val);
            },
            Expression::Unit => {
                write!(&mut self.out, "()").ok();
            }
        }
    }
}