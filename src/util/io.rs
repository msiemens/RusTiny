//! File I/O related helpers

use std::fs::File;
use std::io::Read;
use std::path::Path;
use driver::fatal;


/// Read a file and return it's contents
pub fn read_file(input_path: &str) -> String {
    let mut file = match File::open(&Path::new(input_path)) {
        Ok(f) => f,
        Err(err) => fatal(format!("Can't open {}: {}", input_path, err))
    };

    let mut bytes = Vec::new();

    match file.read_to_end(&mut bytes) {
        Ok(..) => {},
        Err(_) => fatal(format!("Can't read {}", input_path))
    };

    match String::from_utf8(bytes) {
        Ok(contents) => return contents,
        Err(_) => fatal(format!("{} is not UTF-8 encoed", input_path))
    }
}
