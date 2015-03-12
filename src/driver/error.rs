use std::io::{self, Write};
use std::old_io;
use ansi_term::Colour::{Red, Yellow};
use driver::codemap::Loc;


fn is_tty() -> bool {
    old_io::stdio::stderr_raw().isatty()
}

pub fn abort(msg: String) -> ! {
    let mut stderr = io::stderr();

    if is_tty() {
        writeln!(&mut stderr, "{}: {}", Red.paint("Error"), msg).ok();
    } else {
        writeln!(&mut stderr, "Error: {}", msg).ok();
    }

    old_io::stdio::set_stderr(Box::new(old_io::util::NullWriter));
    panic!();
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

pub fn warn(msg: String, source: Loc) {
    let mut stderr = io::stderr();

    if is_tty() {
        writeln!(&mut stderr, "{} in line {}:{}: {}", Yellow.paint("Warning"), source.line, source.col, msg).ok();
    } else {
        writeln!(&mut stderr, "Warning in line {}:{}: {}", source.line, source.col, msg).ok();
    }
}