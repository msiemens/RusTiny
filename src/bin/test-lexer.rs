//! Parse a file and pretty print the AST

extern crate env_logger;
extern crate rustiny;

use std::env;
use rustiny::util::read_file;


#[cfg(not(test))]
fn main() {
    env_logger::init().unwrap();

    // Parse arguments
    let input_file = &env::args().collect::<Vec<_>>()[1];

    // Read source file
    let source = read_file(input_file);

    // Set up
    rustiny::front::setup();

    // Parsing
    let mut lexer = rustiny::front::Lexer::new(&source, input_file);
    println!("{:?}", lexer.tokenize());
}