//! The Driver
//!
//! # Motivation
//!
//! The driver is responsible for coordinating all steps of compilation.

use front;
use middle;
use back;

pub use self::session::session;


pub mod codemap;
mod error;
pub mod interner;
pub mod symbol_table;
mod session;


#[derive(Clone, Copy, Debug, PartialEq, RustcDecodable)]
pub enum CompilationTarget {
    Ir,
    Asm,
    Bin
}


/// The main entry point for compiling a file
pub fn compile_input(source: String, input_file: String, target: CompilationTarget) {
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

    if target == CompilationTarget::Ir {
        println!("{}", ir);
        return
    }

    // Phase 4: Optimization

    // --- Back end -------------------------------------------------------------
    // Phase 5: Machine code generation
    // middle::calculate_liveness(&ir);
    let assembly = back::select_instructions(&ir);

    if target == CompilationTarget::Asm {
        println!("{}", assembly);
        return
    }

    // Phase 6: Register allocation
    let assembly = back::allocate_regs(assembly);

    // Phase 7: Assembly optimization

    println!("{}", assembly);
}