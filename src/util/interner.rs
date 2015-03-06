use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;
use ast::Ident;


// --- String interner ----------------------------------------------------------
// Inspired by: http://doc.rust-lang.org/src/syntax/util/interner.rs.html

#[derive(Clone, PartialEq, Hash, PartialOrd)]
pub struct RcStr {
    string: Rc<String>,
}

impl RcStr {
    pub fn new(string: &str) -> RcStr {
        RcStr {
            string: Rc::new(string.to_string()),
        }
    }
}

impl Eq for RcStr {}

impl Ord for RcStr {
    fn cmp(&self, other: &RcStr) -> Ordering {
        self[..].cmp(&other[..])
    }
}

impl fmt::Debug for RcStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Debug;
        self[..].fmt(f)
    }
}

impl fmt::Display for RcStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Display;
        self[..].fmt(f)
    }
}

impl Borrow<str> for RcStr {
    fn borrow(&self) -> &str {
        &self.string[..]
    }
}

impl Deref for RcStr {
    type Target = str;

    fn deref(&self) -> &str { &self.string[..] }
}


pub struct Interner {
    map: RefCell<HashMap<RcStr, Ident>>,
    vec: RefCell<Vec<RcStr>>
}

impl Interner {
    pub fn new() -> Interner {
        Interner {
            map: RefCell::new(HashMap::new()),
            vec: RefCell::new(Vec::new())
        }
    }

    pub fn intern(&self, val: &str) -> Ident {
        let mut map = self.map.borrow_mut();
        let mut vec = self.vec.borrow_mut();

        if let Some(&idx) = map.get(val) {
            return idx;
        }

        let idx = Ident(vec.len());
        let val = RcStr::new(val);
        map.insert(val.clone(), idx);
        vec.push(val);

        idx
    }

    pub fn get(&self, ident: Ident) -> RcStr {
        let Ident(idx) = ident;
        self.vec.borrow()[idx].clone()
    }
}