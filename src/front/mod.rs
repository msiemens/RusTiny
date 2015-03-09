/// The front end: parsing + semantic analysis

mod lexer;
mod parser;
mod tokens;


pub use self::lexer::Lexer;
pub use self::parser::Parser;
pub use self::tokens::Token;


pub fn setup() {
	// Load all keywords into the interning table
    tokens::intern_keywords();
}