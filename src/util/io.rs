//! File I/O related helpers

use driver::session;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// Read a file and return it's contents
pub fn read_file(input_path: &str) -> String {
    let mut file = match File::open(&Path::new(input_path)) {
        Ok(f) => f,
        Err(err) => {
            fatal!("Can't open {}: {}", input_path, err);
            session().abort()
        }
    };

    let mut bytes = Vec::new();

    match file.read_to_end(&mut bytes) {
        Ok(..) => {}
        Err(_) => {
            fatal!("Can't read {}", input_path);
            session().abort()
        }
    };

    match String::from_utf8(bytes) {
        Ok(contents) => return contents,
        Err(_) => {
            fatal!("{} is not UTF-8 encoed", input_path);
            session().abort()
        }
    };
}

pub fn write_file(output_path: &str, contents: &str) {
    let mut file = match File::create(&Path::new(output_path)) {
        Ok(f) => f,
        Err(err) => {
            fatal!("Can't open {}: {}", output_path, err);
            session().abort()
        }
    };

    match file.write_all(contents.as_bytes()) {
        Ok(..) => {}
        Err(_) => {
            fatal!("Can't write {}", output_path);
            session().abort()
        }
    };
}
