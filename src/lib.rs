#![deny(unused_features)]
#![deny(deprecated)]
#![warn(unused_variables)]
#![warn(unused_imports)]
#![warn(dead_code)]
#![warn(missing_copy_implementations)]
//#![warn(missing_docs)]

extern crate ansi_term;
extern crate term;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate rustc_serialize;

#[macro_use]
pub mod macros;

pub mod back;
pub mod driver;
pub mod front;
pub mod middle;
pub mod util;
