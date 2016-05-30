#![feature(plugin)]
#![feature(slice_patterns)]
#![plugin(clippy)]

#![deny(unused_features)]
#![deny(deprecated)]
#![warn(unused_variables)]
#![warn(unused_imports)]
#![warn(dead_code)]
#![warn(missing_copy_implementations)]
//#![warn(missing_docs)]
#![allow(doc_markdown)]
#![allow(new_without_default)]
#![allow(new_without_default_derive)]
#![allow(while_let_loop)]  // Clippy is buggy here with multiple if lets (see Manishearth/rust-clippy#771) 

extern crate ansi_term;
extern crate term;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate rustc_serialize;

#[macro_use] pub mod macros;

pub mod front;
pub mod middle;
pub mod back;
pub mod driver;
pub mod util;
