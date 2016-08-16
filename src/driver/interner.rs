//! The string interner
//!
//! # Motivation
//!
//! In a `RusTiny` source file, many strings are repeated often (`fn`, variable names).
//! Instead of having these in memory multiple times (e.g. one `fn` per function),
//! we store them in a string intern pool and use a unique ID instead.
//!
//! One could say that the Ident (which stores the string ID) acts as a pointer
//! into the intern pool, which isn't far from the truth.
//!
//! An important effect of string interning is that interned strings are immutable.
//! However, I've didn't need to mutate interned strings up to now.
//!
//! # Implementation notes
//!
//! The implementation is adapted from https://github.com/rust-lang/rust/blob/79dd393a4f144fa5e6f81c720c782de3175810d7/src/libsyntax/util/interner.rs
//! Strings are stored in the thread local storage.

use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::mem;
use std::ops::Deref;
use std::rc::Rc;
use driver::session;


/// An identifier refering to an interned string
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Ident(pub usize);

impl Ident {
    pub fn new(s: &str) -> Ident {
        session().interner.intern(s)
    }
}

/// Allows the ident's name to be accessed by dereferencing (`*ident`)
impl Deref for Ident {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe { mem::transmute(&*(session().interner.resolve(*self))) }
    }
}



/// An string stored in the interner
#[derive(Clone, PartialEq, Hash, PartialOrd)]
pub struct InternedString {
    string: Rc<String>,
}

impl InternedString {
    pub fn new(string: &str) -> InternedString {
        InternedString {
            string: Rc::new(string.to_owned()),
        }
    }
}

impl Eq for InternedString {}

impl Ord for InternedString {
    fn cmp(&self, other: &InternedString) -> Ordering {
        self[..].cmp(&other[..])
    }
}

impl fmt::Debug for InternedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self[..].fmt(f)
    }
}

impl fmt::Display for InternedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self[..].fmt(f)
    }
}

// Needed for indexing a HashMap<InternedString, _> by a &str
impl Borrow<str> for InternedString {
    fn borrow(&self) -> &str { &self.string[..] }
}

// *interned_string becomes a &str
impl Deref for InternedString {
    type Target = str;

    fn deref(&self) -> &str { &self.string[..] }
}


/// The interner itself
pub struct Interner {
    map: RefCell<HashMap<InternedString, Ident>>,
    vec: RefCell<Vec<InternedString>>
}

impl Interner {
    /// Create a new interner instance
    pub fn new() -> Interner {
        Interner {
            map: RefCell::new(HashMap::new()),
            vec: RefCell::new(Vec::new())
        }
    }

    /// Intern a string (if not already interned) and return its identifier
    ///
    /// N.B. This should be the only place where `Ident`s are created
    pub fn intern(&self, val: &str) -> Ident {
        let mut map = self.map.borrow_mut();
        let mut vec = self.vec.borrow_mut();

        if let Some(&idx) = map.get(val) {
            // String is already stored
            return idx;
        }

        // Intern the string and return its Ident
        let idx = Ident(vec.len());
        let val = InternedString::new(val);
        map.insert(val.clone(), idx);
        vec.push(val);

        idx
    }

    /// Get the string value of an identifier
    ///
    /// # Panics
    ///
    /// Panics if the Ident does not exist
    pub fn resolve(&self, ident: Ident) -> InternedString {
        let Ident(idx) = ident;
        self.vec.borrow()[idx].clone()
    }
}