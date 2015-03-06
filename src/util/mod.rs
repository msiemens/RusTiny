use ansi_term::Colour::{Red, Yellow};
use std::fs::File;
use std::io::Read;
use std::old_io;
use std::path::Path;

pub use self::interner::Interner;


mod interner;


/// Read a file and return it's contents
pub fn read_file(input_path: &str) -> String {
    let mut file = match File::open(&Path::new(input_path)) {
        Ok(f) => f,
        Err(err) => panic!("Can't open {}: {}", input_path, err)
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(contents) => contents,
        Err(_) => panic!("Can't read {}", input_path)
    };

    contents
}


// --- Warnings and errors ------------------------------------------------------

#[macro_export]
macro_rules! fatal(
    ($msg:expr, $($args:expr),* ; $stmt:expr) => {
        {
            use assembler::util::fatal;
            fatal(format!($msg, $($args),*), &$stmt.location)
        }
    };

    ($msg:expr ; $stmt:expr) => {
        {
            use std::borrow::ToOwned;
            ::assembler::util::fatal($msg.to_owned(), &$stmt.location)
        }
    };
);

pub fn fatal(msg: String, source: usize) -> ! {
    println!("{} in line {}: {}", Red.paint("Error"), source, msg);

    old_io::stdio::set_stderr(Box::new(old_io::util::NullWriter));
    panic!();
}


#[macro_export]
macro_rules! warn(
    ($msg:expr, $($args:expr),* ; $stmt:expr ) => {
        ::assembler::util::warn(format!($msg, $($args),*), &$stmt.location)
    }
);

pub fn warn(msg: String, source: usize) {
    println!("{} in line {}: {}", Yellow.paint("Warning"), source, msg);
}