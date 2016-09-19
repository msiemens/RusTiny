//! The Driver
//!
//! # Motivation
//!
//! The driver is responsible for coordinating all steps of compilation.

use std::fmt::Write;
use front;
use middle;
use back;
use util::write_file;

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


macro_rules! print_or_write {
    ($output_file:expr, $s:expr) => {
        if let Some(output_file) = $output_file {
            let mut s = String::new();
            write!(s, "{}", $s).unwrap();
            write_file(output_file, &s);
        } else {
            println!("{}", $s);
        }
    };
}


/// The main entry point for compiling a file
pub fn compile_input(source: &str, input_file: &str, output_file: Option<&str>, target: CompilationTarget) {
    // --- Front end ------------------------------------------------------------
    // Set up
    front::setup();

    // Phase 1: Lexical & syntactical analysis
    let lexer = front::Lexer::new(source, input_file);
    let mut parser = front::Parser::new(lexer);
    let ast = parser.parse();

    // Phase 2: Analysis passes (semantic checking, type checking)
    front::semantic_checks(&ast);
    front::type_check(&ast);

    // --- Middle end -----------------------------------------------------------
    // Phase 3: Intermediate code generation
    let ir = middle::ir::translate(&ast);

    if target == CompilationTarget::Ir {
        print_or_write!(output_file, ir);
        return
    }

    use util;
    let mut s = String::new();
    write!(s, "{}", ir).unwrap();
    util::write_file(".debug.ir", &s);

    // Phase 4: Optimization

    // --- Back end -------------------------------------------------------------

    // Phase 5: Machine code generation
    // middle::calculate_liveness(&ir);


    let assembly = back::select_instructions(&ir);

    // Phase 6: Register allocation
    let assembly = back::allocate_regs(assembly);

    // Phase 7: Assembly optimization

    //    if target == CompilationTarget::Asm {
    print_or_write!(output_file, assembly);
    //    }

    // Phase 8: Linking
}