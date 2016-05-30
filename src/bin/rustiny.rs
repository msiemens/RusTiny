//! The main executable: Start the compilation

extern crate docopt;
extern crate env_logger;
extern crate rustc_serialize;
extern crate rustiny;

use docopt::Docopt;
use rustiny::driver::{compile_input, CompilationTarget};
use rustiny::util::read_file;

static USAGE: &'static str = "
Usage: rustiny [options] <input>
       rustiny --help

Options:
    --target TYPE   Configure the output that rustiny will produce.
                    Valid values: bin, asm, ir.
    -o <output>     Write output to <output>.
    --help          Show this screen.
";


#[derive(RustcDecodable, Debug)]
struct Args {
    arg_input: String,
    flag_target: Option<CompilationTarget>
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
    compile_input(source, args.arg_input,
                  args.flag_target.unwrap_or(CompilationTarget::Bin));
}