//! TODO: Docs
#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate docopt;
extern crate env_logger;
extern crate rustc_serialize;
extern crate rustiny;

use rustiny::util::{read_file, write_file};

docopt!(Args derive Debug, "
Usage: rustiny-rulecomp [options] <input>
       rustiny-rulecomp --help

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

    // Compile rules
    let rules = rustiny::back::compile_rules(&source, &args.arg_input);

    if !args.flag_o.is_empty() {
        write_file(&args.flag_o, &rules);
    } else {
        println!("{}", &rules)
    }
}