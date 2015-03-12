//! Coordinating all the steps of compilation: The Driver (tm)

use std::rc::Rc;
use front;
use self::codemap::Codemap;
use self::interner::Interner;

pub use self::error::{fatal, fatal_at, warn_at};


pub mod codemap;
mod error;
mod interner;
pub mod symbol_table;


/// The current compiling session
///
/// Every member has to use interior mutability so the current session can be
/// stored in the thread local storage.
pub struct Session {
    pub codemap: Codemap,
    pub interner: Interner,
}


/// Get a reference to the thread local session object
pub fn session() -> Rc<Session> {
    thread_local! {
        static SESSION: Rc<Session> = Rc::new(Session {
            codemap: Codemap::new(),
            interner: Interner::new(),
        })
    };

    SESSION.with(|o| o.clone())
}


pub fn compile_input(source: String, input_file: String) {
    // --- Front end ------------------------------------------------------------
    // Set up
    front::setup();

    // Phase 1: Lexical & syntactical analysis
    let lexer = front::Lexer::new(&source, &input_file);
    let mut parser = front::Parser::new(lexer);
    let mut ast = parser.parse();

    // Phase 2: Analysis passes (semantic checking, type checking)
    front::semantic_checks(&mut ast);

    // --- Middle end -----------------------------------------------------------
    // Phase 3: Intermediate code generation
    // Phase 4: Optimization

    // --- Back end -------------------------------------------------------------
    // Phase 5: Register allocation
    // Phase 6: Machine code generation
    // Phase 7: Assembly optimization
}