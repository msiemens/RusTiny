//! Error reporting

// IDEA: Cache errors in the session and have an abort_if_errors() that
//       is called every now and then.
// IDEA: Show the source line and underline the offending token

use std::env;
use std::io::{self, Write};
use std::process;
use ansi_term::Colour::Red;
use term;
use driver;
use driver::codemap::{BytePos, Loc};
use front::ast::{Node, Span};


pub trait HasSourceLocation {
    fn loc(&self) -> Loc;
}

impl<'a, T> HasSourceLocation for &'a Node<T> {
    fn loc(&self) -> Loc {
        self.span.loc()
    }
}

impl HasSourceLocation for Span {
    fn loc(&self) -> Loc {
        driver::session().codemap.resolve(BytePos(self.pos))
    }
}

impl HasSourceLocation for Loc {
    fn loc(&self) -> Loc {
        *self
    }
}


fn colors_enabled() -> bool {
    if env::var_os("COLORED_OUTPUT").and_then(|s| s.into_string().ok()) == Some("off".into()) {
        return false;
    }

    term::stderr().map_or(false, |t| {
        t.supports_attr(term::Attr::ForegroundColor(term::color::RED))
        && t.supports_attr(term::Attr::ForegroundColor(term::color::YELLOW))
    })
}


/// Abort compilation
pub fn abort() -> ! {
    process::exit(1)
}


/// Helper for printing the `Error` string
/// If stderr is not redirected, the string will be colored
fn print_error(stderr: &mut io::Stderr) {
    if colors_enabled() {
        write!(stderr, "{}", Red.paint("Error")).ok();
    } else {
        write!(stderr, "Error").ok();
    }
}

/// Report a fatal error
pub fn fatal<S: AsRef<str>>(msg: S) {
    let mut stderr = io::stderr();

    print_error(&mut stderr);
    writeln!(&mut stderr, ": {}", msg.as_ref()).ok();
}


/// Report a fatal error at a source location
pub fn fatal_at<S: AsRef<str>>(msg: S, source: Loc) {
    let mut stderr = io::stderr();

    print_error(&mut stderr);
    writeln!(&mut stderr, " in line {}:{}: {}", source.line, source.col, msg.as_ref()).ok();
}


/*
/// Helper for printing the `Warning` string
/// If stderr is not redirected, the string will be colored
fn print_warning(stderr: &mut io::Stderr) {
    if !colors_enabled() {
        write!(stderr, "{}", Yellow.paint("Warning")).ok();
    } else {
        write!(stderr, "Error").ok();
    }
}

/// Report a warning
pub fn warn(msg: String, source: Loc) {
    let mut stderr = io::stderr();

    print_warning(&mut stderr);
    writeln!(&mut stderr, ": {}", msg).ok();
}


/// Report a warning at a source location
pub fn warn_at(msg: String, source: Loc) {
    let mut stderr = io::stderr();

    print_warning(&mut stderr);
    writeln!(&mut stderr, " in line {}:{}: {}", source.line, source.col, msg).ok();
}
*/