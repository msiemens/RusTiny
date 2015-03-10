//! The codemap
//!
//! The codemap allows to map source locations (Spans) to the original line
//! and column number.

use std::cell::RefCell;


/// A source location (used for error reporting)
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
    pub fn new_line(&self, char_pos: u32) {
        self.lines.borrow_mut().push(char_pos)
    }

    /// Get the source location of an offset
    pub fn resolve(&self, char_pos: u32) -> Loc {
        let lines = self.lines.borrow();

        let line = lines
            .iter()
            .position(|p| *p >= char_pos)
            .unwrap_or(lines.len() - 1);
        let offset = lines[line];

        Loc {
            line: line as u32 + 1,
            col: char_pos - offset + 1
        }
    }
}


mod test {
    use super::Codemap;

    #[test]
    fn test_basic() {
        let cm = Codemap::new();
        cm.new_line(16);
        let loc = cm.resolve(31);
        assert_eq!(loc.line, 2);
        assert_eq!(loc.col, 16);
    }

    #[test]
    fn test_first_line() {
        let cm = Codemap::new();
        let loc = cm.resolve(31);
        assert_eq!(loc.line, 1);
        assert_eq!(loc.col, 32);
    }
}