#![feature(plugin)]
#![feature(slice_patterns)]
#![plugin(clippy)]

#![deny(unused_features)]
#![deny(deprecated)]
#![warn(unused_variables)]
#![warn(unused_imports)]
#![warn(dead_code)]
#![warn(missing_copy_implementations)]
#![allow(new_without_default)]
#![allow(while_let_loop)]  // Clippy is buggy here with multiple if lets
//#![warn(missing_docs)]

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
