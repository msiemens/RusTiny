#![feature(collections)]
#![feature(fs)]
#![feature(io)]
#![feature(old_io)]
#![feature(path)]
#![feature(plugin)]
#![plugin(docopt_macros)]

#![allow(dead_code)]

extern crate ansi_term;
extern crate docopt;
extern crate env_logger;
extern crate "rustc-serialize" as rustc_serialize;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

mod ast;
mod front;
mod middle;
mod back;
mod driver;
mod util;

use docopt::Docopt;
use util::read_file;

docopt!(Args derive Debug, "
Usage: tinyc [options] <input>
       tinyc --help

Options:
    -o <output>     Write output to <output>
    --help          Show this screen
");


#[cfg(not(test))]
fn main() {
    env_logger::init().unwrap();

    // Parse arguments
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    // Read source file
    let source = read_file(&args.arg_input);

    // Start compilation
    driver::compile_input(source, args.arg_input);
}