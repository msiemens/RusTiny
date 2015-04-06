//! The main executable: Start the compilation

extern crate docopt;
extern crate env_logger;
extern crate rustc_serialize;
extern crate rustiny;

use docopt::Docopt;
use rustiny::util::read_file;

static USAGE: &'static str = "
Usage: rustiny [options] <input>
       rustiny --help

Options:
    --ir            Emit IR only
    -o <output>     Write output to <output>
    --help          Show this screen
";


#[derive(RustcDecodable, Debug)]
struct Args {
    arg_input: String,
    flag_ir: bool
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

    // Start compilation
    rustiny::driver::compile_input(source, args.arg_input, args.flag_ir);
}