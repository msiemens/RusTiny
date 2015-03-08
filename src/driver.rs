/// Coordinating all the steps of compilation: The Driver (tm)

use util::{get_interner, PrettyPrinter};
use front;


pub fn compile_input(source: String, input_file: String) {
    // --- Front end ------------------------------------------------------------
    // Set up
    front::setup();

    // Phase 1: Lexical & syntactical analysis
    let lexer = front::Lexer::new(&source, &input_file);
    let mut parser = front::Parser::new(lexer);
    let ast = parser.parse();

    // For debugging the lexer/parser:
    PrettyPrinter::print(&ast);

    // Phase 2: Analysis passes (semantic checking, type checking)
    //  Semantic checks:
    // - impl only for datatypes
    // - Left hand of assignment is variable
    // - fn main() is present
    // - Scope checking
    //      - Every variable only defined once
    //      - No usage of undeclared variables
    // - Type checking
    //      - Expression in if/while is a boolean

    // --- Middle end -----------------------------------------------------------
    // Phase 3: Intermediate code generation
    // Phase 4: Optimization

    // --- Back end -------------------------------------------------------------
    // Phase 5: Register allocation
    // Phase 6: Machine code generation
    // Phase 7: Assembly optimization
}