//! The Driver
//!
//! # Motivation
//!
//! The driver is responsible for coordinating all steps of compilation.

use front;
use middle;
use back;

pub use self::session::session;
//pub use self::error::abort;


pub mod codemap;
mod error;
mod interner;
pub mod symbol_table;
pub mod session;


/// The main entry point for compiling a file
pub fn compile_input(source: String, input_file: String, ir_only: bool) {
    // --- Front end ------------------------------------------------------------
    // Set up
    front::setup();

    // Phase 1: Lexical & syntactical analysis
    let lexer = front::Lexer::new(&source, &input_file);
    let mut parser = front::Parser::new(lexer);
    let ast = parser.parse();

    // Phase 2: Analysis passes (semantic checking, type checking)
    front::semantic_checks(&ast);
    front::type_check(&ast);

    // --- Middle end -----------------------------------------------------------
    // Phase 3: Intermediate code generation
    let ir = middle::ir::translate(&ast);

    if ir_only {
        println!("{}", ir);
        return
    }

    // Phase 4: Optimization

    // --- Back end -------------------------------------------------------------
    // Phase 5: Machine code generation
    let assembly = back::select_instructions(&ir);

    // Phase 6: Register allocation
    // Phase 7: Assembly optimization

    println!("{:?}", assembly);
}