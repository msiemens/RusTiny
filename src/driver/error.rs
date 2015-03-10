use std::io::{self, Write};
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


fn is_tty() -> bool {
    old_io::stdio::stderr_raw().isatty()
}


pub fn fatal(msg: String, source: Loc) -> ! {
    let mut stderr = io::stderr();

    if is_tty() {
        writeln!(&mut stderr, "{} in line {}:{}: {}", Red.paint("Error"), source.line, source.col, msg).ok();
    } else {
        writeln!(&mut stderr, "Error in line {}:{}: {}", source.line, source.col, msg).ok();
    }

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
    let mut stderr = io::stderr();

    if is_tty() {
        writeln!(&mut stderr, "{} in line {}:{}: {}", Yellow.paint("Warning"), source.line, source.col, msg).ok();
    } else {
        writeln!(&mut stderr, "Warning in line {}:{}: {}", source.line, source.col, msg).ok();
    }
}