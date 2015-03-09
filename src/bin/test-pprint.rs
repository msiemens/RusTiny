//! Parse a file and pretty print the AST

#![feature(io)]
#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate docopt;
extern crate env_logger;
extern crate "rustc-serialize" as rustc_serialize;
extern crate rustiny;

use std::io;
use docopt::Docopt;
use rustiny::util::{read_file, PrettyPrinter};


docopt!(Args derive Debug, "
Usage: pprint <input>
       pprint --help

Options:
    --help          Show this screen
");


#[cfg(not(test))]
fn main() {
    env_logger::init().unwrap();

    // Parse arguments
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    // Read source file
    let source = read_file(&args.arg_input);

    // Set up
    rustiny::front::setup();

    // Parsing
    let lexer = rustiny::front::Lexer::new(&source, &args.arg_input);
    let mut parser = rustiny::front::Parser::new(lexer);
    let ast = parser.parse();

    // Pretty printing
    PrettyPrinter::print(&ast, &mut io::stdout());
}