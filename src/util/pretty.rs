//! A pretty printer
//!
//! Prints a parsed AST so that parsing it again would result in the same AST
//! again.

use ast::*;


pub struct PrettyPrinter<'a> {
    indent: u32,
    program: &'a Program
}

impl<'a> PrettyPrinter<'a> {
    pub fn print(program: &'a Program) {
        PrettyPrinter {
            indent: 0,
            program: program
        }.print_program();
    }

    fn print_indent(&mut self) {
        for _ in 0..(self.indent * 4) {
            print!(" ");
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
                ref body,
                local_vars: _
            } => self.print_function(&name, &bindings, ret_ty, &body)
        }
    }

    fn print_static(&mut self, binding: &Binding, value: &Value) {
        println!("static {:?} = {:?};", binding, value)
    }

    fn print_constant(&mut self, binding: &Binding, value: &Value) {
        println!("const {:?} = {:?};", binding, value)
    }

    fn print_function(&mut self,
                      name: &str,
                      bindings: &[Node<Binding>],
                      ret_ty: &Type,
                      body: &Block)
    {
        println!("");
        print!("fn {}({}) -> {:?} ",
               name,
               bindings
                  .iter()
                  .map(|b| format!("{:?}", b))
                  .collect::<Vec<_>>()
                  .connect(", "),
               ret_ty);
        self.print_block(body)
    }

    fn print_block(&mut self, block: &Block) {
        //self.print_indent();
        println!("{{");

        self.indent += 1;

        for stmt in &block.stmts {
            self.print_statement(&stmt);
        }

        match block.expr {
            Some(ref expr) => {
                self.print_indent();
                self.print_expression(expr);
                println!("");
            },
            None => {}
        }

        self.indent -= 1;

        self.print_indent();
        print!("}}");
    }

    fn print_statement(&mut self, stmt: &Statement) {
        self.print_indent();

        match *stmt {
            Statement::Declaration { ref binding, ref value } => {
                print!("let {:?} = ", binding);
                self.print_expression(value);
                println!(";")
            },
            Statement::Expression { ref val } => {
                self.print_expression(val);
                println!(";")
            }
        }
    }

    fn print_expression(&mut self, expr: &Expression) {
        match *expr {
            Expression::Group(ref expr) => {
                print!("(");
                self.print_expression(expr);
                print!(")");
            },
            Expression::Call { ref func, ref args } => {
                self.print_expression(func);
                print!("(");
                for arg in args {
                    self.print_expression(arg);
                    print!(", ");
                }
                print!(")");
            },
            Expression::Infix { ref op, ref lhs, ref rhs } => {
                self.print_expression(lhs);
                print!(" {:?} ", op);
                self.print_expression(rhs);
            },
            Expression::Prefix { ref op, ref item } => {
                print!("{:?}", op);
                self.print_expression(item);
            },
            Expression::Literal { ref val } => {
                print!("{:?}", val);
            },
            Expression::Variable { ref name } => {
                print!("{:?}", name);
            },
            Expression::If { ref cond, ref conseq, ref altern } => {
                print!("if ");
                self.print_expression(cond);
                print!(" ");
                self.print_block(conseq);
                match *altern {
                    Some(ref b) => {
                        print!(" else ");
                        self.print_block(b);
                    },
                    None => {}
                }
            },
            Expression::While { ref cond, ref body } => {
                print!("while ");
                self.print_expression(cond);
                print!(" ");
                self.print_block(body);
            },
            Expression::Assign { ref lhs, ref rhs } => {
                self.print_expression(lhs);
                print!(" = ");
                self.print_expression(rhs);
            },
            Expression::AssignOp { ref op, ref lhs, ref rhs } => {
                self.print_expression(lhs);
                print!(" {:?}= ", op);
                self.print_expression(rhs);
            },
            Expression::Break => {
                print!("break");
            },
            Expression::Return { ref val } => {
                print!("return ");

                match *val {
                    Some(ref expr) => self.print_expression(expr),
                    None => {}
                }
            }
        }
    }
}