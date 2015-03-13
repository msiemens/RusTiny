//! Error reporting

// IDEA: Cache errors in the session and have an abort_if_errors() that
//       is called every now and then.
// IDEA: Show the source line and underline the offending token

use std::io::{self, Write};
use std::old_io;
use ansi_term::Colour::Red;
use ast::{Node, Span};
use driver;
use driver::codemap::Loc;


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
        driver::session().codemap.resolve(self.pos)
    }
}

impl HasSourceLocation for Loc {
    fn loc(&self) -> Loc {
        *self
    }
}


/// Helper that checks whether stderr is redirected
fn stderr_is_redirected() -> bool {
    !old_io::stdio::stderr_raw().isatty()
}

/// Helper for printing the `Error` string
/// If stderr is not redirected, the string will be colored
fn print_error(stderr: &mut io::Stderr) {
    if !stderr_is_redirected() {
        write!(stderr, "{}", Red.paint("Error")).ok();
    } else {
        write!(stderr, "Error").ok();
    }
}

/// Report a fatal error
// FIXME: Add a macro with format!(...)
pub fn fatal(msg: String) -> ! {
    let mut stderr = io::stderr();

    print_error(&mut stderr);
    writeln!(&mut stderr, ": {}", msg).ok();

    old_io::stdio::set_stderr(Box::new(old_io::util::NullWriter));
    panic!();
}


/// Report a fatal error at a source location
pub fn fatal_at<L: HasSourceLocation>(msg: String, loc: L) -> ! {
    let mut stderr = io::stderr();

    let source = loc.loc();

    print_error(&mut stderr);
    writeln!(&mut stderr, " in line {}:{}: {}", source.line, source.col, msg).ok();

    old_io::stdio::set_stderr(Box::new(old_io::util::NullWriter));
    panic!();
}


/*
/// Helper for printing the `Warning` string
/// If stderr is not redirected, the string will be colored
fn print_warning(stderr: &mut io::Stderr) {
    if !stderr_is_redirected() {
        write!(stderr, "{}", Yellow.paint("Warning")).ok();
    } else {
        write!(stderr, "Error").ok();
    }
}

/// Report a warning
pub fn warn<L: HasSourceLocation>(msg: String, loc: L) {
    let mut stderr = io::stderr();

    let source = loc.loc();

    print_warning(&mut stderr);
    writeln!(&mut stderr, ": {}", msg).ok();
}


/// Report a warning at a source location
pub fn warn_at<L: HasSourceLocation>(msg: String, loc: L) {
    let mut stderr = io::stderr();

    let source = loc.loc();

    print_warning(&mut stderr);
    writeln!(&mut stderr, " in line {}:{}: {}", source.line, source.col, msg).ok();
}
*/