//! The Driver
//!
//! # Motivation
//!
//! The driver is responsible for coordinating all steps of compilation.

use front;

pub use self::session::session;
//pub use self::error::abort;


pub mod codemap;
mod error;
mod interner;
pub mod symbol_table;
pub mod session;


/// The main entry point for compiling a file
pub fn compile_input(source: String, input_file: String) {
    // --- Front end ------------------------------------------------------------
    // Set up
    front::setup();

    // Phase 1: Lexical & syntactical analysis
    let lexer = front::Lexer::new(&source, &input_file);
    let mut parser = front::Parser::new(lexer);
    let ast = parser.parse();

    // Phase 2: Analysis passes (semantic checking, type checking)
    let symbol_table = front::semantic_checks(&ast);
    front::type_check(&ast, &symbol_table);

    // --- Middle end -----------------------------------------------------------
    // Phase 3: Intermediate code generation
    // Phase 4: Optimization

    // --- Back end -------------------------------------------------------------
    // Phase 5: Machine code generation
    // Phase 6: Register allocation
    // Phase 7: Assembly optimization
}