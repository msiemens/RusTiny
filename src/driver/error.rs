use std::old_io;
use ansi_term::Colour::{Red, Yellow};
use driver::codemap::Loc;

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

pub fn fatal(msg: String, source: Loc) -> ! {
    println!("{} in line {}:{}: {}", Red.paint("Error"), source.line, source.col, msg);

    old_io::stdio::set_stderr(Box::new(old_io::util::NullWriter));
    panic!();
}


#[macro_export]
macro_rules! warn(
    ($msg:expr, $($args:expr),* ; $stmt:expr ) => {
        ::assembler::util::warn(format!($msg, $($args),*), &$stmt.location)
    }
);

pub fn warn(msg: String, source: Loc) {
    println!("{} in line {}:{}: {}", Yellow.paint("Warning"), source.line, source.col, msg);
}