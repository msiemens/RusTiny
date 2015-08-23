//! All tokens that RusTiny understands

use std::fmt;
use ::Ident;
use driver;

// --- List of tokens -----------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Token {
    Bang,
    Dollar,
    Percent,
    Zero,
    Equal,
    Plus,
    Minus,
    Asterisk,
    DoubleDot,

    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    FatArrow,

    Keyword(Keyword),
    Ident(Ident),
    Literal(Ident),

    EOF,
    PLACEHOLDER
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Token::*;

        match *self {
            Bang                => write!(f, "!"),
            Dollar              => write!(f, "$"),
            Percent             => write!(f, "%"),
            Zero                => write!(f, "0"),
            Equal               => write!(f, "="),
            Plus                => write!(f, "+"),
            Minus               => write!(f, "-"),
            Asterisk            => write!(f, "*"),
            DoubleDot           => write!(f, ".."),

            LParen              => write!(f, "("),
            RParen              => write!(f, ")"),
            LBracket            => write!(f, "["),
            RBracket            => write!(f, "]"),
            LBrace              => write!(f, "{{"),
            RBrace              => write!(f, "}}"),
            Comma               => write!(f, ","),
            Semicolon           => write!(f, ";"),
            FatArrow            => write!(f, "=>"),

            Keyword(ref kw)     => write!(f, "{}", kw),
            Ident(id)           => write!(f, "{}", id),
            Literal(lit)        => write!(f, "{}", lit),

            EOF                 => write!(f, "EOF"),
            PLACEHOLDER         => write!(f, "PLACEHOLDER"),
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

        /// Load all keywords into the interner
        // TODO: Call this in init!
        pub fn intern_keywords() {
            $( driver::session().interner.intern($name); )*
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
}