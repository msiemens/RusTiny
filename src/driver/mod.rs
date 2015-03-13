//! The Driver
//!
//! # Motivation
//!
//! The driver is responsible for coordinating all steps of compilation.
//!
//! # The Session
//!
//! Some modules are needed throughout almost all parts of the code. This
//! could be implemented by passing a Session struct to every step. But
//! this results in a rather ugly design and might lead to problems when
//! different modules run interleaved (e.g. lexing and parsing). Thus,
//! we store the current Session in the thread local storage and provide
//! a method for accessing it (`driver::session()`).
//!
//! In `driver::session()` we only can hand out an immutable reference
//! to the current Session. Thus, to modify the current session, its members
//! have to rely on interior mutability (methods looking immutable but
//! actually modifying the session, implemented by a RefCell).
//! This isn't a really clean solution either, but it's better than the
//! alternatives IMO.

use std::rc::Rc;
use front;
use self::codemap::Codemap;
use self::interner::Interner;


pub use self::error::{fatal, fatal_at};


pub mod codemap;
mod error;
mod interner;
pub mod symbol_table;


/// The current compiling session
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