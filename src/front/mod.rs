//! The front end: parsing + semantic analysis

mod lexer;
mod parser;
mod tokens;
mod semck;


pub use self::lexer::Lexer;
pub use self::parser::Parser;
pub use self::tokens::Token;
pub use self::semck::check as semantic_checks;


pub fn setup() {
	// Load all keywords into the interning table
    tokens::intern_keywords();
}