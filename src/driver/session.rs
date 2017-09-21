//! The Session
//!
//! # Motivation
//!
//! Some modules are needed throughout almost all parts of the code. This
//! could be implemented by passing a Session struct to every step. But
//! this results in a rather ugly design and might lead to problems when
//! different modules run interleaved (e.g. lexing and parsing). Thus,
//! we store the current Session in the thread local storage and provide
//! a method for accessing it (`driver::session()`).
//!
//! In `driver::session::session()` we only can hand out an immutable reference
//! to the current Session. Thus, to modify the current session, its members
//! have to rely on interior mutability (methods looking immutable but
//! actually modifying the session, implemented by a `RefCell`).
//! This isn't a really clean solution either, but it's better than the
//! alternatives IMO.

use std::cell::RefCell;
use std::rc::Rc;
use driver::codemap::Codemap;
use driver::error::{self, HasSourceLocation};
use driver::interner::Interner;
use driver::symbol_table::SymbolTable;


/// The current compiling session
pub struct Session {
    pub codemap: Codemap,
    pub interner: Interner,
    pub errors: RefCell<bool>,
    pub symbol_table: SymbolTable,
}

impl Session {
    /// Print an error with a source location
    pub fn span_err<T: HasSourceLocation>(&self, msg: String, loc: T) {
        error::fatal_at(&*msg, loc.loc());
        *self.errors.borrow_mut() = true;
    }

    /// Print an error
    pub fn err(&self, msg: String) {
        error::fatal(&*msg);
        *self.errors.borrow_mut() = true;
    }

    /// Abort complation if any errors occured
    pub fn abort_if_errors(&self) {
        if *self.errors.borrow() {
            error::abort();
        }
    }

    /// Abort complation
    pub fn abort(&self) -> ! {
        error::abort()
    }
}


/// Get a reference to the thread local session object
pub fn session() -> Rc<Session> {
    thread_local! {
        static SESSION: Rc<Session> = Rc::new(Session {
            codemap: Codemap::new(),
            interner: Interner::new(),
            errors: RefCell::new(false),
            symbol_table: SymbolTable::new()
        })
    };

    SESSION.with(|o| Rc::clone(o))
}
