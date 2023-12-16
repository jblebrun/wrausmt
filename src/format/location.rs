/// The location of a token in a source file, represented as a `line` and `pos`
/// (column).
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Location {
    pub line: u32,
    pub pos:  u32,
}

impl Location {
    /// Advance the location to the next line, resetting the position.
    pub fn nextline(&mut self) {
        self.line += 1;
        self.pos = 0;
    }

    /// Advance the location to the next position.
    pub fn nextchar(&mut self) {
        self.pos += 1;
    }

    pub fn advanceby(&mut self, amt: usize) {
        self.pos += amt as u32;
    }
}
