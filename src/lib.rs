#![feature(collections)]
#![feature(io)]
#![feature(old_io)]
#![feature(path)]
#![feature(std_misc)]

#![deny(unused_imports)]
#![deny(unused_variables)]
#![deny(dead_code)]
// #![warn(missing_docs)]

extern crate ansi_term;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

#[macro_use] pub mod macros;

pub mod ast;
pub mod front;
pub mod middle;
pub mod back;
pub mod driver;
pub mod util;