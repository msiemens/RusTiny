use std::collections::HashMap;
use std::rc::Rc;
use ast::{Ident, Symbol};
use util::Interner;
use front;


type SymbolTable = HashMap<Ident, Symbol>;


/// Get a reference to the thread local interner
pub fn get_interner() -> Rc<Interner> {
    thread_local! {
        static INTERNER: Rc<Interner> = Rc::new(Interner::new())
    };

    INTERNER.with(|o| o.clone())
}

/// Get a reference to the thread local symbol table
pub fn get_symbol_table() -> Rc<SymbolTable> {
    thread_local! {
        static SYMBOL_TABLE: Rc<HashMap<Ident, Symbol>> = Rc::new(HashMap::new())
    };

    SYMBOL_TABLE.with(|o| o.clone())
}

pub fn compile_input(source: String, input_file: String) {
    // --- Front end ------------------------------------------------------------
    // Set up
    front::setup();

    // Phase 1: Lexical & syntactical analysis
    let lexer = front::Lexer::new(&source, &input_file);
    //println!("{:?}", lexer.tokenize());
    let mut parser = front::Parser::new(lexer);
    let ast = parser.parse();

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