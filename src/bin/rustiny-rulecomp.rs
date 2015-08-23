//! TODO: Docs

extern crate docopt;
extern crate env_logger;
extern crate rustc_serialize;
extern crate rustiny;

use docopt::Docopt;
use rustiny::util::read_file;

static USAGE: &'static str = "
Usage: rustiny-rulecomp <input> <output>
       rustiny-rulecomp --help

Options:
    --help          Show this screen
";


#[derive(RustcDecodable, Debug)]
struct Args {
    arg_input: String,
    arg_output: String,
}


#[cfg(not(test))]
fn main() {
    env_logger::init().unwrap();

    // Parse arguments
    let args: Args = Docopt::new(USAGE)
                             .and_then(|d| d.decode())
                             .unwrap_or_else(|e| e.exit());

    // Read source file
    let source = read_file(&args.arg_input);

    // Compile rules
    let rules = rustiny::back::compile_rules(&source, &args.arg_input);
    // TODO: Print to file
    println!("{}", rules);
}