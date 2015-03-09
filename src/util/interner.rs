//! The string interner
//!
//! Inspired by: http://doc.rust-lang.org/src/syntax/util/interner.rs.html
//! Stores strings in a thread local hashmap and assigns an ID to them
//! for easier use.

use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;
use ast::Ident;


/// Get a reference to the thread local interner
pub fn get_interner() -> Rc<Interner> {
    thread_local! {
        static INTERNER: Rc<Interner> = Rc::new(Interner::new())
    };

    INTERNER.with(|o| o.clone())
}


/// An interned string
#[derive(Clone, PartialEq, Hash, PartialOrd)]
pub struct InternedString {
    string: Rc<String>,
}

impl InternedString {
    pub fn new(string: &str) -> InternedString {
        InternedString {
            string: Rc::new(string.to_string()),
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
        use std::fmt::Debug;
        self[..].fmt(f)
    }
}

impl fmt::Display for InternedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Display;
        self[..].fmt(f)
    }
}

impl Borrow<str> for InternedString {
    fn borrow(&self) -> &str { &self.string[..] }
}

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

    /// Intern a string and return the new identifier
    pub fn intern(&self, val: &str) -> Ident {
        let mut map = self.map.borrow_mut();
        let mut vec = self.vec.borrow_mut();

        if let Some(&idx) = map.get(val) {
            return idx;
        }

        let idx = Ident(vec.len());
        let val = InternedString::new(val);
        map.insert(val.clone(), idx);
        vec.push(val);

        idx
    }

    /// Get the string value of an identifier
    pub fn resolve(&self, ident: Ident) -> InternedString {
        let Ident(idx) = ident;
        self.vec.borrow()[idx].clone()
    }
}