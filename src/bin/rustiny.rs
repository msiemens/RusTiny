//! The main executable: Start the compilation

extern crate clap;
extern crate env_logger;
extern crate rustc_serialize;
extern crate rustiny;

use clap::{Arg, App};

use rustiny::driver::{compile_input, CompilationTarget};
use rustiny::util::read_file;


#[cfg(not(test))]
fn main() {
    env_logger::init().unwrap();

    // Parse arguments
    let app = App::new("rustiny")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Markus SIemens <markus@m-siemens.de>")

        .arg(Arg::with_name("target")
            .short("t")
            .long("target")
            .value_name("TYPE")
            .help("Sets which type of output to generate")
            .possible_values(&["bin", "asm", "ir"])
            .default_value("bin")
            .takes_value(true))

        .arg(Arg::with_name("output")
            .short("o")
            .value_name("OUTPUT")
            .help("Sets the output file"))

        .arg(Arg::with_name("INPUT")
            .help("Sets the file to compile")
            .required(true)
            .index(1));
    let args = app.get_matches();

    // Read source file
    let input_file = args.value_of("INPUT").unwrap();
    let source = read_file(input_file);

    let target = match args.value_of("target").unwrap() {
        "bin" => CompilationTarget::Bin,
        "asm" => CompilationTarget::Asm,
        "ir" => CompilationTarget::Ir,
        s => panic!(format!("Invalid target: {}", s))
    };

    // Start compilation
    compile_input(source, input_file.into(), target);
}