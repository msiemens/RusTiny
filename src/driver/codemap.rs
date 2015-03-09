use std::cell::RefCell;


/// A source location (used for error reporting)
pub struct Loc {
    /// The (1-based) line number
    pub line: u32,
    /// The (0-based) column offset
    pub col: u32
}


pub struct Codemap {
    // Mapping lineno -> start index
    lines: RefCell<Vec<u32>>
}

impl Codemap {
    pub fn new() -> Codemap {
        let mut lines = Vec::new();
        lines.push(0);

        Codemap { lines: RefCell::new(lines) }
    }

    pub fn new_line(&self, char_pos: u32) {
        self.lines.borrow_mut().push(char_pos)
    }

    pub fn resolve(&self, char_pos: u32) -> Loc {
        let lines = self.lines.borrow();
        println!("lines: {:?}", lines);
        println!("pos: {:?}", char_pos);

        let line = lines
            .iter()
            .position(|p| *p >= char_pos)
            .unwrap_or(lines.len() - 1);
        let offset = lines[line];

        Loc {
            line: line as u32 + 1,
            col: char_pos - offset
        }
    }
}