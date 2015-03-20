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

// TODO: Let the codemap own the source string

use std::cell::RefCell;
use std::ops::{Add, Sub};


#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct BytePos(pub u32);

impl BytePos {
    pub fn as_int(&self) -> u32 {
        let BytePos(i) = *self;
        i
    }
}

impl Add for BytePos {
    type Output = BytePos;

    fn add(self, rhs: BytePos) -> BytePos {
        BytePos((self.as_int() + rhs.as_int()) as u32)
    }
}

impl Add<u32> for BytePos {
    type Output = BytePos;

    fn add(self, rhs: u32) -> BytePos {
        BytePos((self.as_int() + rhs) as u32)
    }
}

impl Sub for BytePos {
    type Output = BytePos;

    fn sub(self, rhs: BytePos) -> BytePos {
        BytePos((self.as_int() - rhs.as_int()) as u32)
    }
}


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
    lines: RefCell<Vec<BytePos>>
}

impl Codemap {
    /// Create a new Codemap instance
    pub fn new() -> Codemap {
        let mut lines = Vec::new();
        lines.push(BytePos(0));

        Codemap { lines: RefCell::new(lines) }
    }

    /// Register the beginning of a new line at a given offset
    pub fn new_line(&self, pos: BytePos) {
        let mut lines = self.lines.borrow_mut();
        let line_count = lines.len();

        assert!(line_count == 0 || (lines[line_count - 1] < pos));
        lines.push(pos)
    }

    /// Get the source location of an offset
    pub fn resolve(&self, mut pos: BytePos) -> Loc {
        let lines = self.lines.borrow();

        debug!("pos: {:?}", pos);
        debug!("lines: {:?}", lines);

        // Binary search for the index at which the offset is <= pos
        let mut lower = 0;
        let mut upper = lines.len();

        while upper - lower > 1 {
            let mid = (lower + upper) / 2;
            let offset = lines[mid];
            if offset > pos {
                // Continue on the left
                upper = mid;
            } else {
                // Continue on the right
                lower = mid;
            }
        }

        let line = lower;
        let offset = lines[line];

        // FIXME: Why is this needed?
        if line == 0 {
            pos = pos + 1;
        }

        debug!("line: {:?}", line);
        debug!("offset: {:?}", offset);

        debug_assert!(pos >= offset,
                      "pos ({:?}) >= offset ({:?})\nlines: {:?}\nline:{:?}",
                      pos, offset, lines, line);

        Loc {
            line: line as u32 + 1,
            col: (pos - offset).as_int()
        }
    }
}