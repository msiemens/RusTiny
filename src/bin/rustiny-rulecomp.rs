//! TODO: Docs
#![feature(plugin)]

extern crate clap;
extern crate env_logger;
extern crate rustc_serialize;
extern crate rustiny;

use clap::{Arg, App};

use rustiny::util::{read_file, write_file};


#[cfg(not(test))]
fn main() {
    env_logger::init().unwrap();

    // Parse arguments
    let app = App::new("rustiny-rulecomp")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Markus SIemens <markus@m-siemens.de>")

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

    // Compile rules
    let rules = rustiny::back::compile_rules(&source, input_file);

    if let Some(output_file) = args.value_of("output") {
        write_file(output_file, &rules);
    } else {
        println!("{}", &rules)
    }
}