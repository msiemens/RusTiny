use std::collections::{HashMap, LinkedList};
use ast::*;
use front::Lexer;
use front::tokens::{Token, Keyword};
use front::parselet::PARSELET_MANAGER;
use util::fatal;


pub struct Parser<'a> {
    location: usize,
    pub token: Token,
    buffer: LinkedList<Token>,
    lexer: Lexer<'a>
}

impl<'a> Parser<'a> {
    pub fn new(mut lx: Lexer<'a>) -> Parser<'a> {
        Parser {
            token: lx.next_token(),
            location: lx.get_source(),
            buffer: LinkedList::new(),
            lexer: lx
        }
    }

    pub fn parse(&mut self) -> Program {
        let mut source = vec![];

        debug!("Starting parsing");

        while self.token != Token::EOF {
            source.push(self.parse_symbol());
        }

        debug!("Parsing finished");

        source
    }

    // --- Error handling -------------------------------------------------------

    fn fatal(&self, msg: String) -> ! {
        fatal(msg, self.location);
    }

    pub fn unexpected_token(&self, expected: Option<&'static str>) -> ! {
        match expected {
            Some(ex) => self.fatal(format!("unexpected token: `{:?}`, expected {:?}", &self.token, ex)),
            None => self.fatal(format!("unexpected token: `{:?}`", &self.token))
        }
    }

    // --- Token processing -----------------------------------------------------

    fn update_location(&mut self) -> usize {
        self.location = self.lexer.get_source();
        self.location.clone()
    }

    pub fn bump(&mut self) {
        self.token = match self.buffer.pop_front() {
            Some(tok) => tok,
            None => self.lexer.next_token()
        };
    }

    pub fn eat(&mut self, tok: Token) -> bool {
        if self.token == tok {
            self.bump();
            true
        } else {
            false
        }
    }

    pub fn expect(&mut self, tok: Token) {
        if !self.eat(tok) {
            self.fatal(format!("expected `{:?}`, found `{:?}`", tok, self.token))
        }
    }

    pub fn look_ahead<F, R>(&mut self, distance: usize, f: F) -> R where F: Fn(Token) -> R {
        if self.buffer.len() < distance {
            for _ in 0 .. distance - self.buffer.len() {
                self.buffer.push_back(self.lexer.next_token());
            }
        }

        f(*self.buffer.iter().nth(distance - 1).unwrap())
    }

    // --- Actual parsing -------------------------------------------------------

    pub fn parse_ident(&mut self) -> Ident {
        let ident = match self.token {
            Token::Ident(id) => id,
            _ => self.unexpected_token(Some("an identifier"))
        };
        self.bump();

        ident
    }

    pub fn parse_literal(&mut self) -> Node<Expression> {
        let value = match self.token {
            Token::Int(i) => Value::Int(i),
            Token::Char(c) => Value::Char(c),
            _ => self.unexpected_token(Some("a literal"))
        };
        self.bump();

        Node::new(Expression::Literal {
            val: value
        })
    }

    fn parse_type(&mut self) -> Type {
        let ident = self.parse_ident();
        let ty: Result<Type, ()> = (*ident).parse();
        match ty {
            Ok(ty) => ty,
            Err(()) => self.unexpected_token(Some("a type"))
        }
    }

    fn parse_binding(&mut self) -> Node<Binding> {
        let name = self.parse_ident();
        self.expect(Token::Colon);
        let ty = self.parse_type();

        Node::new(Binding {
            ty: ty,
            name: name
        })
    }

    fn parse_block(&mut self) -> Node<Block> {
        println!("--- parsing a block");

        self.expect(Token::LBrace);

        let mut stmts = vec![];
        let mut expr = None;

        loop {
            match self.token {
                Token::Keyword(Keyword::Let) => {
                    stmts.push(self.parse_declaration());
                    continue
                },
                Token::Keyword(Keyword::If) => {
                    stmts.push(Node::new(Statement::Expression {
                        val: Box::new(self.parse_if())
                    }));
                    continue
                },
                Token::Keyword(Keyword::While) => {
                    stmts.push(Node::new(Statement::Expression {
                        val: Box::new(self.parse_while())
                    }));
                    continue
                },
                _ => {}
            }

            // Try parsing an expression
            println!("--- parsing an expression");

            while self.eat(Token::Semicolon) {}
            if self.token == Token::RBrace { break }

            let maybe_expr = self.parse_expression();

            println!("--- done parsing an expression");

            if self.eat(Token::Semicolon) {
                // It's actually a statement
                stmts.push(Node::new(Statement::Expression {
                    val: Box::new(maybe_expr)
                }));
            } else {
                // It's the last expr
                expr = Some(maybe_expr);
                break;
            }
        }

        self.expect(Token::RBrace);

        println!("--- done parsing a block");

        Node::new(Block {
            stmts: stmts,
            expr: expr.map(|e| Box::new(e))
        })
    }

    // --- Parsing: Statements --------------------------------------------------

    fn parse_declaration(&mut self) -> Node<Statement> {
        self.expect(Token::Keyword(Keyword::Let));
        let binding = self.parse_binding();
        self.expect(Token::Eq);
        let value = self.parse_expression();
        self.expect(Token::Semicolon);

        Node::new(Statement::Declaration {
            binding: Box::new(binding),
            value: Box::new(value)
        })
    }

    // --- Parsing: Expressions -------------------------------------------------

    pub fn parse_expression(&mut self) -> Node<Expression> {
        self.parse_expression_with_precedence(0)
    }

    pub fn parse_expression_with_precedence(&mut self, precedence: u32) -> Node<Expression> {
        match self.token {
            Token::Keyword(Keyword::If) => self.parse_if(),
            Token::Keyword(Keyword::While) => self.parse_while(),
            Token::Keyword(Keyword::Return) => {
                self.bump();
                let val = if let Token::RBrace = self.token {
                    None
                } else {
                    Some(self.parse_expression())
                };

                Node::new(Expression::Return {
                    val: val.map(|e| Box::new(e))
                })
            },
            Token::Keyword(Keyword::Break) => {
                self.bump();
                Node::new(Expression::Break)
            },
            _ => self.prett_parser(0)
        }
    }

    fn current_precedence(&self) -> u32 {
        match PARSELET_MANAGER.lookup_infix(self.token) {
            Some(p) => p.precedence(),
            None => 0
        }
    }

    fn prett_parser(&mut self, precedence: u32) -> Node<Expression> {
        debug!("prefix: current token: {:?}", self.token);

        let token = self.token;
        self.bump();

        let pparselet = match PARSELET_MANAGER.lookup_prefix(token) {
            Some(p) => p,
            None => self.unexpected_token(Some("a prefix expression"))
        };

        debug!("prefix: parselet: {:?}", pparselet.name());

        let mut left = pparselet.parse(self, token);

        debug!("prefix: done");

        while precedence < self.current_precedence() {
            debug!("infix: current token: {:?}", self.token);

            let token = self.token;
            let iparselet = PARSELET_MANAGER.lookup_infix(token).unwrap();
            debug!("infix: parselet: {:?}", iparselet.name());

            self.bump();
            left = iparselet.parse(self, left, token);
            debug!("infix: done");
        }

        left
    }

    fn parse_if(&mut self) -> Node<Expression> {
        self.bump();
        let cond = self.parse_expression();
        let conseq = self.parse_block();
        let altern = if self.eat(Token::Keyword(Keyword::Else)) {
            Some(self.parse_block())
        } else {
            None
        };

        Node::new(Expression::If {
            cond: Box::new(cond),
            conseq: Box::new(conseq),
            altern: altern.map(|b| Box::new(b))
        })
    }

    fn parse_while(&mut self) -> Node<Expression> {
        self.bump();
        let cond = self.parse_expression();
        let body = self.parse_block();

        Node::new(Expression::While {
            cond: Box::new(cond),
            body: Box::new(body)
        })
    }

    // --- Parsing: Symbols -----------------------------------------------------

    fn parse_fn(&mut self) -> Node<Symbol> {
        // k_fn IDENT LPAREN (binding COMMA)* binding? RPAREN (RARROW TYPE)? block
        self.expect(Token::Keyword(Keyword::Fn));
        let ident = self.parse_ident();
        self.expect(Token::LParen);

        let mut bindings = vec![];
        while self.token != Token::RParen {
            bindings.push(self.parse_binding());
            if !self.eat(Token::Comma) {
                break
            }
        }

        self.expect(Token::RParen);

        let ret_ty = if self.eat(Token::RArrow) {
            self.parse_type()
        } else {
            Type::Unit
        };

        let body = self.parse_block();

        Node::new(Symbol::Function{
            name: ident,
            bindings: bindings,
            ret_ty: ret_ty,
            body: Box::new(body),
            local_vars: HashMap::new()
        })
    }

    fn parse_static(&mut self) -> Node<Symbol> {
        // k_static binding EQ literal
        self.expect(Token::Keyword(Keyword::Static));
        let binding = self.parse_binding();
        self.expect(Token::Eq);
        let value = match self.parse_literal().unwrap() {
            Expression::Literal { val } => val,
            _ => panic!("shouldn't happen")
        };

        Node::new(Symbol::Static {
            binding: Box::new(binding),
            value: value
        })
    }

    fn parse_const(&mut self) -> Node<Symbol> {
        // k_const binding EQ literal
        self.expect(Token::Keyword(Keyword::Const));
        let binding = self.parse_binding();
        self.expect(Token::Eq);
        let value = match self.parse_literal().unwrap() {
            Expression::Literal { val } => val,
            _ => panic!("shouldn't happen")
        };

        Node::new(Symbol::Constant {
            binding: Box::new(binding),
            value: value
        })
    }

    fn parse_symbol(&mut self) -> Node<Symbol> {
        let symbol = match self.token {
            Token::Keyword(Keyword::Fn) => self.parse_fn(),
            Token::Keyword(Keyword::Static) => self.parse_static(),
            Token::Keyword(Keyword::Const) => self.parse_const(),
            Token::Keyword(Keyword::Impl) => unimplemented!(),

            _ => self.unexpected_token(Some("a symbol"))
        };

        symbol
    }

}