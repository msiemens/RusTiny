//! The parser: transform a stream of tokens into an Abstract Syntax Tree
//!
//! The parser uses a straight forward recursive descent strategy
//! (each grammar rule has a corresponding parser method, e.g. `parse_symbol`).
//! For expressions a Pratt parser is used (see http://journal.stuffwithstuff.com/2011/03/19/pratt-parsers-expression-parsing-made-easy/
//! for a great introduction).
//!
//! # The grammar (pseudo EBNF)
//!
//! ```ignore
//! # AST
//! program:        comment | (symbol comment?)*
//! symbol:         function | static | constant | impl
//!
//! function:       k_fn IDENT LPAREN (binding COMMA)* binding? RPAREN (RARROW TYPE)? block
//! static:         k_static binding EQ literal
//! constant:       k_const binding EQ literal
//! impl:           k_impl LBRACE function* RBRACE
//!
//! binding:        IDENT COLON TYPE
//! block:          LBRACE (declaration | expression SEMICOLON)* expr? RBRACE
//!
//! declaration:    k_let binding EQ expression
//! expression:     call
//!                 | binary
//!                 | unary
//!                 | literal
//!                 | if
//!                 | while
//!                 | assign
//!                 | assign_op
//!                 | break
//!                 | return
//!                 | variable
//!
//! literal:        BOOL | INT | CHAR
//! variable:       IDENT
//! assign:         expression EQ expression
//! assign_op:      expression BINOP EQ expression
//! return:         k_return (expression)?
//! call:           IDENT LPAREN (expr COMMA)* expr? RPAREN
//! group:          LPAREN expr RPAREN
//! infix:          expression BINOP expression
//! prefix:         UNOP expression
//! if:             k_if expression block (k_else block)?
//! while:          k_while expression block
//! break:          k_break
//!
//!
//! # Tokens
//! BINOP:      '+' | '-' | '*' | '/' | '%' | '&&' | '||' | '^' | '&' | '|' |
//!             '<<' | '>>' | '==' | '<' | '<=' | '!=' | '>=' | '>' | '**'
//! UNOP:       '-' | '!'
//! IDENT:      [a-Z]+ ( '_' | [a-Z] | [0-9]+ )+
//! TYPE:       [a-Z]+ ( '_' | [a-Z] | [0-9]+ )+
//! LPAREN:     '('
//! RPAREN:     ')'
//! LBRACE:     '{'
//! RBRACE:     '}'
//! COMMA:      ','
//! COLON:      ':'
//! SEMICOLON:  ';'
//! RARROW:     '->'
//! EQ:         '='
//!
//! BOOL:       'true' | 'false'
//! INT:        [0-9]+
//! CHAR:       '\'' ( [a-z] | [A-Z] | '\n' ) '\''
//! ```

use std::collections::HashMap;
use ast::*;
use front::Lexer;
use front::tokens::{Token, Keyword};
use front::parser::parselet::PARSELET_MANAGER;
use util::fatal;


// Parselets for the Pratt parser
mod parselet;


pub struct Parser<'a> {
    location: usize,
    token: Token,
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    // --- The public API -------------------------------------------------------

    /// Create a new parser instance
    pub fn new(mut lx: Lexer<'a>) -> Parser<'a> {
        Parser {
            token: lx.next_token(),  // Initialize with first token
            location: lx.get_source(),
            lexer: lx
        }
    }

    /// Process all tokens and create an AST
    pub fn parse(&mut self) -> Program {
        let mut source = vec![];

        debug!("Starting parsing");

        // A program is a list of symbols
        while self.token != Token::EOF {
            source.push(self.parse_symbol());
        }

        debug!("Parsing finished");

        source
    }

    // --- Error handling -------------------------------------------------------

    /// Stop compiling because of a fatal error
    fn fatal(&self, msg: String) -> ! {
        fatal(msg, self.location);
    }

    /// Stop compiling because of an unexpected token
    fn unexpected_token(&self, expected: Option<&'static str>) -> ! {
        match expected {
            Some(ex) => self.fatal(format!("unexpected token: `{:?}`, expected {}",
                                   &self.token, ex)),
            None => self.fatal(format!("unexpected token: `{:?}`", &self.token))
        }
    }

    // --- Token processing -----------------------------------------------------

    /// Move along to the next token
    fn bump(&mut self) {
        self.token = self.lexer.next_token();
    }

    /// Try consuming a token, return `true` on succes
    fn eat(&mut self, tok: Token) -> bool {
        if self.token == tok {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Try consuming a token, quit with a fatal error otherwise
    fn expect(&mut self, tok: Token) {
        if !self.eat(tok) {
            self.fatal(format!("expected `{:?}`, found `{:?}`", tok, self.token))
        }
    }

    // --- Parse tokens ---------------------------------------------------------

    /// Parse an identifier
    fn parse_ident(&mut self) -> Ident {
        let ident = match self.token {
            Token::Ident(id) => id,
            _ => self.unexpected_token(Some("an identifier"))
        };
        self.bump();

        ident
    }

    /// Parse a literal
    fn parse_literal(&mut self) -> Node<Expression> {
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

    /// Parse a builitin type
    fn parse_type(&mut self) -> Type {
        let ident = self.parse_ident();
        let ty: Result<Type, ()> = (*ident).parse();
        match ty {
            Ok(ty) => ty,
            Err(()) => self.unexpected_token(Some("a type"))
        }
    }

    // --- Parse helpers --------------------------------------------------------

    /// Parse a binding
    fn parse_binding(&mut self) -> Node<Binding> {
        // Grammar: IDENT COLON TYPE

        let name = self.parse_ident();
        self.expect(Token::Colon);
        let ty = self.parse_type();

        Node::new(Binding {
            ty: ty,
            name: name
        })
    }

    /// Parse a block of expressions
    fn parse_block(&mut self) -> Node<Block> {
        // Grammar: LBRACE (statement SEMICOLON)* expr? RBRACE

        // Blocks are funny things in Rust. They contain:
        // - a list of semicolon-separated statements,
        // - and an optional expression that acts as the block's value.
        // It requires a little work to get this right.

        debug!("parsing a block");

        self.expect(Token::LBrace);

        let mut stmts = vec![];
        let mut expr = None;

        // Parse all statements
        loop {
            // Special cases: declarations, if's and while's
            match self.token {
                Token::Keyword(Keyword::Let) => {
                    stmts.push(self.parse_declaration());
                    continue
                },

                // If and While expressions can appear as statements ...
                // ... without a trainling semicolon!
                Token::Keyword(Keyword::If) => {
                    let if_expr = self.parse_if();

                    stmts.push(Node::new(Statement::Expression {
                        val: Box::new(if_expr)
                    }));
                    continue
                },

                Token::Keyword(Keyword::While) => {
                    let while_expr = self.parse_while();

                    stmts.push(Node::new(Statement::Expression {
                        val: Box::new(while_expr)
                    }));
                    continue
                },

                _ => {}
            }

            while self.eat(Token::Semicolon) {
                // Eat all semicolons that are remaining
            }

            if self.token == Token::RBrace {
                // We've reached the end of the block already
                break
            }

            // Parse the expression
            debug!("parsing an expression");
            let maybe_expr = self.parse_expression();
            debug!("done parsing an expression");

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

        debug!("done parsing a block");

        Node::new(Block {
            stmts: stmts,
            expr: expr.map(|e| Box::new(e))
        })
    }

    // --- Parsing: Statements --------------------------------------------------

    fn parse_declaration(&mut self) -> Node<Statement> {
        // Grammar: k_let binding EQ expression

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
    // Using the Pratt parser technique

    /// Parse an arbitrary expression
    fn parse_expression(&mut self) -> Node<Expression> {
        self.parse_expression_with_precedence(0)
    }

    /// Parse an expression with a specified precedence
    fn parse_expression_with_precedence(&mut self, precedence: u32) -> Node<Expression> {
        match self.token {
            Token::Keyword(Keyword::If) => self.parse_if(),
            Token::Keyword(Keyword::While) => self.parse_while(),
            Token::Keyword(Keyword::Return) => {
                self.bump();
                // Parse the return value
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
            _ => self.pratt_parser(precedence)
        }
    }

    /// The current token's infix precedence
    fn current_precedence(&self) -> u32 {
        match PARSELET_MANAGER.lookup_infix(self.token) {
            Some(p) => p.precedence(),
            None => 0
        }
    }

    /// Entry point for the Pratt parser
    fn pratt_parser(&mut self, precedence: u32) -> Node<Expression> {
        debug!("prefix: current token: {:?}", self.token);

        // Look up the prefix parselet
        let pparselet = match PARSELET_MANAGER.lookup_prefix(self.token) {
            Some(p) => {
                debug!("prefix: parselet: {:?}", p.name());
                p
            },
            None => self.unexpected_token(Some("a prefix expression"))
        };

        // Parse the prefix expression
        let token = self.token;
        self.bump();
        let mut left = pparselet.parse(self, token);

        debug!("prefix: done");

        while precedence < self.current_precedence() {
            debug!("infix: current token: {:?}", self.token);

            // Look up the infix parselet (unwrapping it!)
            let iparselet = PARSELET_MANAGER.lookup_infix(self.token).unwrap();
            debug!("infix: parselet: {:?}", iparselet.name());

            // Parse the infix expression
            let token = self.token;
            self.bump();
            left = iparselet.parse(self, left, token);

            debug!("infix: done");
        }

        left
    }

    fn parse_if(&mut self) -> Node<Expression> {
        // Grammar: k_if expression block (k_else block)?

        self.expect(Token::Keyword(Keyword::If));

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
        // Grammar: k_while expression block

        self.expect(Token::Keyword(Keyword::While));
        let cond = self.parse_expression();
        let body = self.parse_block();

        Node::new(Expression::While {
            cond: Box::new(cond),
            body: Box::new(body)
        })
    }

    // --- Parsing: Symbols -----------------------------------------------------

    fn parse_fn(&mut self) -> Node<Symbol> {
        // Grammar:  k_fn IDENT LPAREN (binding COMMA)* binding? RPAREN (RARROW TYPE)? block

        // Parse `fn <name>`
        self.expect(Token::Keyword(Keyword::Fn));
        let ident = self.parse_ident();

        self.expect(Token::LParen);

        // Parse the expected arguments
        let mut bindings = vec![];
        while self.token != Token::RParen {
            bindings.push(self.parse_binding());
            if !self.eat(Token::Comma) {
                break
            }
        }

        self.expect(Token::RParen);

        // Parse the return type
        let ret_ty = if self.eat(Token::RArrow) {
            self.parse_type()
        } else {
            Type::Unit
        };

        // Parse the body
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
        // Grammar: k_static binding EQ literal

        self.expect(Token::Keyword(Keyword::Static));

        let binding = self.parse_binding();

        self.expect(Token::Eq);

        let value = match self.parse_literal().unwrap() {
            Expression::Literal { val } => val,
            _ => panic!("shouldn't happen")
        };

        self.expect(Token::Semicolon);

        Node::new(Symbol::Static {
            binding: Box::new(binding),
            value: value
        })
    }

    fn parse_const(&mut self) -> Node<Symbol> {
        // Grammar: k_const binding EQ literal

        self.expect(Token::Keyword(Keyword::Const));

        let binding = self.parse_binding();

        self.expect(Token::Eq);

        let value = match self.parse_literal().unwrap() {
            Expression::Literal { val } => val,
            _ => panic!("shouldn't happen")
        };

        self.expect(Token::Semicolon);

        Node::new(Symbol::Constant {
            binding: Box::new(binding),
            value: value
        })
    }

    fn parse_symbol(&mut self) -> Node<Symbol> {
        // Grammar: function | static | constant | impl

        let symbol = match self.token {
            Token::Keyword(Keyword::Fn) => self.parse_fn(),
            Token::Keyword(Keyword::Static) => self.parse_static(),
            Token::Keyword(Keyword::Const) => self.parse_const(),
            Token::Keyword(Keyword::Impl) => unimplemented!(),  // TODO: Implement

            _ => self.unexpected_token(Some("a symbol"))
        };

        symbol
    }

}