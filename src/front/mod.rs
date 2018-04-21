//! The front end: parsing + semantic analysis

pub mod ast;
mod lexer;
mod parser;
mod semck;
mod tokens;
mod typeck;

pub use self::lexer::Lexer;
pub use self::parser::Parser;
pub use self::semck::run as semantic_checks;
pub use self::tokens::Token;
pub use self::typeck::run as type_check;

pub fn setup() {
    // Load all keywords into the interning table
    tokens::Keyword::setup();
}
