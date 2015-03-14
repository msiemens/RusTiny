#![feature(collections)]
#![feature(std_misc)]
#![feature(set_panic)]

#![deny(unused_imports)]
#![deny(unused_features)]
#![deny(unused_variables)]
#![deny(dead_code)]
#![deny(deprecated)]
// #![warn(missing_docs)]

extern crate ansi_term;
extern crate term;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

#[macro_use] pub mod macros;

pub mod ast;
pub mod front;
pub mod middle;
pub mod back;
pub mod driver;
pub mod util;