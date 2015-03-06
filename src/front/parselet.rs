use std::collections::HashMap;
use ast::*;
use front::tokens::{Token, TokenType, Keyword};
use front::parser::Parser;


enum Associativity {
    Left,
    Right
}

enum Precedence {
    Call        = 13,
    Prefix      = 12,
    Exponent    = 11,
    Product     = 10,
    Sum         = 9,
    Shift       = 8,
    BitAnd      = 7,
    BitXor      = 6,
    BitOr       = 5,
    Compare     = 4,
    And         = 3,
    Or          = 2,
    Assignment  = 1
}


lazy_static! {
    pub static ref PARSELET_MANAGER: ParseletManager = ParseletManager::new();
}


pub struct ParseletManager {
    prefix: HashMap<TokenType, Box<PrefixParselet + Sync>>,
    infix: HashMap<TokenType, Box<InfixParselet + Sync>>
}

impl ParseletManager {
    pub fn new() -> ParseletManager {
        ParseletManager {
            prefix: HashMap::new(),
            infix:  HashMap::new()
        }.init()
    }

    pub fn lookup_prefix(&self, token: Token) -> Option<&PrefixParselet> {
        self.prefix.get(&token.ty()).map(|p| &**p)
    }

    pub fn lookup_infix(&self, token: Token) -> Option<&InfixParselet> {
        self.infix.get(&token.ty()).map(|p| &**p)
    }

    fn init(mut self) -> Self {
        use self::Associativity::*;

        // Prefix parselets
        self.register_literal(TokenType::Literal);
        self.register_prefix(TokenType::Ident,  IdentParselet);
        self.register_prefix(TokenType::UnOp,   PrefixOperatorParselet);
        self.register_prefix(TokenType::LParen, GroupParselet);

        // Infix parselets
        self.register_binop(TokenType::BinOp(BinOp::Add),    Precedence::Sum,      Left);
        self.register_binop(TokenType::BinOp(BinOp::Sub),    Precedence::Sum,      Left);
        self.register_binop(TokenType::BinOp(BinOp::Mul),    Precedence::Product,  Left);
        self.register_binop(TokenType::BinOp(BinOp::Div),    Precedence::Product,  Left);
        self.register_binop(TokenType::BinOp(BinOp::Mod),    Precedence::Product,  Left);
        self.register_binop(TokenType::BinOp(BinOp::Pow),    Precedence::Exponent, Right);
        self.register_binop(TokenType::BinOp(BinOp::And),    Precedence::And,      Left);
        self.register_binop(TokenType::BinOp(BinOp::Or),     Precedence::Or,       Left);
        self.register_binop(TokenType::BinOp(BinOp::BitXor), Precedence::BitXor,   Left);
        self.register_binop(TokenType::BinOp(BinOp::BitAnd), Precedence::BitAnd,   Left);
        self.register_binop(TokenType::BinOp(BinOp::BitOr),  Precedence::BitOr,    Left);
        self.register_binop(TokenType::BinOp(BinOp::Shl),    Precedence::Shift,    Left);
        self.register_binop(TokenType::BinOp(BinOp::Shr),    Precedence::Shift,    Left);
        self.register_binop(TokenType::BinOp(BinOp::Lt),     Precedence::Compare,  Left);
        self.register_binop(TokenType::BinOp(BinOp::Le),     Precedence::Compare,  Left);
        self.register_binop(TokenType::BinOp(BinOp::Ne),     Precedence::Compare,  Left);
        self.register_binop(TokenType::BinOp(BinOp::Ge),     Precedence::Compare,  Left);
        self.register_binop(TokenType::BinOp(BinOp::Gt),     Precedence::Compare,  Left);
        self.register_binop(TokenType::BinOp(BinOp::EqEq),   Precedence::Compare,  Left);
        self.register_infix(TokenType::Eq,                   AssignParselet);
        self.register_infix(TokenType::LParen,               CallParselet);

        self
    }

    fn register_literal(&mut self, t: TokenType) {
        self.prefix.insert(t, Box::new(LiteralParselet));
    }

    fn register_prefix<P: PrefixParselet + Sync + 'static>(&mut self, t: TokenType, p: P) {
        self.prefix.insert(t, Box::new(p));
    }

    fn register_binop(&mut self,
                      t: TokenType,
                      precedence: Precedence,
                      assoc: Associativity)
    {
        self.infix.insert(t, Box::new(BinaryOperatorParselet {
            preced: precedence as u32,
            assoc: assoc
        }));
    }

    fn register_infix<P: InfixParselet + Sync + 'static>(&mut self, t: TokenType, p: P) {
        self.infix.insert(t, Box::new(p));
    }
}


// --- Prefix Parselets ---------------------------------------------------------

pub trait PrefixParselet {
    fn parse(&self, parser: &mut Parser, token: Token) -> Node<Expression>;
    fn name(&self) -> &'static str;
}

macro_rules! define_prefix(
    ($name:ident: fn parse($parser:ident, $token:ident) -> Node<Expression> $body:block) => {
        pub struct $name;

        impl PrefixParselet for $name {
            #[allow(unused_variables)]
            fn parse(&self, $parser: &mut Parser, $token: Token) -> Node<Expression> {
                $body
            }

            fn name(&self) -> &'static str { stringify!($name) }
        }
    }
);

define_prefix!(IdentParselet:
    fn parse(parser, token) -> Node<Expression> {
        let ident = match token {
            Token::Ident(id) => id,
            _ => parser.unexpected_token(Some("an identifier"))
        };

        Node::new(Expression::Variable { name: ident })
    }
);

define_prefix!(LiteralParselet:
    fn parse(parser, token) -> Node<Expression> {
        let value = match token {
            Token::Int(i) => Value::Int(i),
            Token::Char(c) => Value::Char(c),
            Token::Keyword(Keyword::True) => Value::Bool(true),
            Token::Keyword(Keyword::False) => Value::Bool(false),
            _ => parser.unexpected_token(Some("a literal"))
        };

        Node::new(Expression::Literal {
            val: value
        })
    }
);

define_prefix!(PrefixOperatorParselet:
    fn parse(parser, token) -> Node<Expression> {
        let operand = parser.parse_expression();
        let op = match token {
            Token::UnOp(op) => op,
            Token::BinOp(BinOp::Sub) => UnOp::Neg,  // FIXME
            _ => parser.unexpected_token(Some("an unary operator"))
        };

        Node::new(Expression::Prefix { op: op, item: Box::new(operand) })
    }
);

define_prefix!(GroupParselet:
    fn parse(parser, token) -> Node<Expression> {
        let expr = parser.parse_expression();
        parser.expect(Token::RParen);

        expr
    }
);


// --- Infix Parselets ----------------------------------------------------------

pub trait InfixParselet {
    fn parse(&self, parser: &mut Parser, left: Node<Expression>, token: Token) -> Node<Expression>;
    fn precedence(&self) -> u32;
    fn name(&self) -> &'static str;
}


pub struct BinaryOperatorParselet {
    preced: u32,
    assoc: Associativity
}

impl InfixParselet for BinaryOperatorParselet {
    fn parse(&self, parser: &mut Parser, left: Node<Expression>, token: Token) -> Node<Expression> {
        use self::Associativity::*;

        let op = match token {
            Token::BinOp(op) => op,
            _ => parser.unexpected_token(Some("a binary operator"))
        };

        let precedence = self.preced - match self.assoc { Left => 0, Right => 1 };

        if parser.token == Token::Eq {
            parser.bump();
            let right = parser.parse_expression_with_precedence(precedence);
            Node::new(Expression::AssignOp {
                op: op,
                lhs: Box::new(left),
                rhs: Box::new(right)
            })
        } else {
            let right = parser.parse_expression_with_precedence(precedence);
            Node::new(Expression::Infix {
                op: op,
                lhs: Box::new(left),
                rhs: Box::new(right)
            })
        }
    }

    fn name(&self) -> &'static str { "BinaryOperatorParselet" }

    fn precedence(&self) -> u32 { self.preced }
}


pub struct AssignParselet;

impl InfixParselet for AssignParselet {
    fn parse(&self, parser: &mut Parser, left: Node<Expression>, token: Token) -> Node<Expression> {
        let right = parser.parse_expression();

        Node::new(Expression::Assign {
            lhs: Box::new(left),
            rhs: Box::new(right)
        })
    }

    fn name(&self) -> &'static str { "AssignParselet" }

    fn precedence(&self) -> u32 { Precedence::Assignment as u32 }
}


pub struct CallParselet;

impl InfixParselet for CallParselet {
    fn parse(&self, parser: &mut Parser, left: Node<Expression>, token: Token) -> Node<Expression> {
        let mut args = vec![];

        while parser.token != Token::RParen {
            args.push(parser.parse_expression());
            if !parser.eat(Token::Comma) {
                break
            }
        }

        parser.expect(Token::RParen);

        Node::new(Expression::Call {
            func: Box::new(left),
            args: args
        })
    }

    fn name(&self) -> &'static str { "CallParselet" }

    fn precedence(&self) -> u32 { Precedence::Call as u32 }
}