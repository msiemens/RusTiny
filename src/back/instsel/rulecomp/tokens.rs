//! All tokens that the rule parser understands

use driver;
use driver::interner::Ident;
use std::fmt;

// --- List of tokens -----------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Token {
    Bang,
    Dollar,
    Percent,
    At,
    Zero,
    Equal,
    Plus,
    Minus,
    Asterisk,
    Dot,
    DoubleDot,

    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Arrow,
    FatArrow,

    Keyword(Keyword),
    Ident(Ident),
    Literal(Ident),

    EOF,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Token::*;

        match *self {
            Bang => write!(f, "!"),
            Dollar => write!(f, "$"),
            Percent => write!(f, "%"),
            At => write!(f, "@"),
            Zero => write!(f, "0"),
            Equal => write!(f, "="),
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Asterisk => write!(f, "*"),
            Dot => write!(f, "."),
            DoubleDot => write!(f, ".."),

            LParen => write!(f, "("),
            RParen => write!(f, ")"),
            LBracket => write!(f, "["),
            RBracket => write!(f, "]"),
            LBrace => write!(f, "{{"),
            RBrace => write!(f, "}}"),
            Comma => write!(f, ","),
            Semicolon => write!(f, ";"),
            Arrow => write!(f, "->"),
            FatArrow => write!(f, "=>"),

            Keyword(ref kw) => write!(f, "{}", kw),
            Ident(ref id) => write!(f, "{}", id),
            Literal(ref lit) => write!(f, "{}", lit),

            EOF => write!(f, "EOF"),
        }
    }
}

// --- List of keywords ---------------------------------------------------------

macro_rules! keywords(
    ($($kw:ident => $name:expr ,)*) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        pub enum Keyword {
            $($kw),*
        }

        impl Keyword {
            /// Load all keywords into the interner
            pub fn setup() {
                $( driver::session().interner.intern($name); )*
            }
        }

        impl fmt::Display for Keyword {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                use self::Keyword::*;

                match *self {
                    $(
                        $kw => write!(f, $name)
                    ),*
                }
            }
        }

        /// Get the keyword a string represents, if possible
        pub fn lookup_keyword(s: &str) -> Option<Keyword> {
            use self::Keyword::*;

            match s {
                $(
                    $name => Some($kw),
                )*
                _ => None
            }
        }
    };
);

keywords! {
    Rules   => "rules",
    If      => "if",
    Add     => "add",
    Sub     => "sub",
    Mul     => "mul",
    Div     => "div",
    Mod     => "mod",
    Pow     => "pow",
    Shl     => "shl",
    Shr     => "shr",
    And     => "and",
    Or      => "or",
    Xor     => "xor",
    Neg     => "neg",
    Not     => "not",
    Cmp     => "cmp",
    Lt      => "lt",
    Le      => "le",
    Eq      => "eq",
    Ne      => "ne",
    Ge      => "ge",
    Gt      => "gt",
    Alloca  => "alloca",
    Load    => "load",
    Store   => "store",
    Call    => "call",
    Ret     => "ret",
    Br      => "br",
    Jmp     => "jmp",
    Byte    => "byte",
    Word    => "word",
    DWord   => "dword",
    QWord   => "qword",
    Ptr     => "ptr",
    Void    => "void",
}
