//! The main executable: Start the compilation

#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate docopt;
extern crate env_logger;
extern crate "rustc-serialize" as rustc_serialize;
extern crate rustiny;

use docopt::Docopt;
use rustiny::util::read_file;

docopt!(Args derive Debug, "
Usage: rustiny [options] <input>
       rustiny --help

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
    rustiny::driver::compile_input(source, args.arg_input);
}