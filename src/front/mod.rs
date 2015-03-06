pub mod lexer;
pub mod parser;
mod parselet;
pub mod tokens;

pub use self::lexer::Lexer;
pub use self::parser::Parser;

pub fn setup() {
    tokens::intern_keywords();
}