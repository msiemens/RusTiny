#![feature(hashmap_hasher)]
#![feature(split_off)]
#![feature(append)]
#![feature(plugin)]

#![plugin(clippy)]

#![allow(needless_return)]
#![allow(needless_lifetimes)]
#![deny(unused_features)]
#![deny(deprecated)]
#![warn(unused_variables)]
#![warn(unused_imports)]
#![warn(dead_code)]
#![warn(missing_copy_implementations)]
//#![warn(missing_docs)]

extern crate ansi_term;
extern crate term;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

#[macro_use] pub mod macros;

pub mod front;
pub mod middle;
pub mod back;
pub mod driver;
pub mod util;


use std::mem;
use std::ops::Deref;
use driver::session;


// FIXME: Maybe move to driver::interner
/// An identifier refering to an interned string
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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
