//! The codemap
//!
//! # Motivation
//!
//! The parser annotates Nodes with `Span`s that map into the source file.
//! This is handy for error reporting, e.g. mentioning the source line
//! or showing a snippet of the offending code. The codemap allows to
//! get the source location of a Span by keeping track of all newlines.
//!
//! Should RusTiny get support for modules, the codemap would be responsible
//! to resolve the file name, too.
//!
//! # Implementation notes
//!
//! The implementation is currently a bit messey. It's not really clear to me
//! what exactly `Codemap::lines` stores and which offset is 0-based and
//! which one is 1-based. But hey, it works!

// TODO: Clear up 0-based vs 1-based offsets
// TODO: Let the codemap own the source string

use std::cell::RefCell;


/// A source location (used for error reporting)
#[derive(Copy)]
pub struct Loc {
    /// The (1-based) line number
    pub line: u32,
    /// The (1-based) column offset
    pub col: u32
}


pub struct Codemap {
    /// Mapping of the line number to the start index
    lines: RefCell<Vec<u32>>
}

impl Codemap {
    /// Create a new Codemap instance
    pub fn new() -> Codemap {
        let mut lines = Vec::new();
        lines.push(0);

        Codemap { lines: RefCell::new(lines) }
    }

    /// Register the beginning of a new line at a given offset
    /// N.B. offset is 1-based
    pub fn new_line(&self, offset: u32) {
        self.lines.borrow_mut().push(offset - 1)
    }

    /// Get the source location of an offset
    /// N.B. char_pos is 0-based!
    pub fn resolve(&self, char_pos: u32) -> Loc {
        let lines = self.lines.borrow();

        debug!("char_pos: {:?}", char_pos);
        debug!("lines: {:?}", lines);

        let line = lines
            .iter()
            .position(|p| *p >= char_pos)  // The first line where offset >= char_pos
            .unwrap_or(lines.len()) - 1;   // Go back one line

        // FIXME: That seems *very* hacky. Can we do better?
        if line == 0 {
            return Loc {
                line: 1,
                col: char_pos + 1
            }
        }

        let offset = lines[line];

        debug!("line: {:?}", line);
        debug!("offset: {:?}", offset);

        debug_assert!(char_pos >= offset,
                      "char_pos ({}) >= offset ({})\nlines: {:?}\nline:{}",
                      char_pos, offset, lines, line);

        Loc {
            line: line as u32 + 1,
            col: char_pos - offset
        }
    }
}