#![feature(collections)]
#![feature(std_misc)]
#![feature(set_stdio)]
#![feature(plugin)]

#![deny(unused_features)]
#![deny(deprecated)]
#![warn(unused_variables)]
#![warn(unused_imports)]
#![warn(dead_code)]
#![warn(missing_copy_implementations)]
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