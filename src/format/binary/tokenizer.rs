use crate::format::Location;
use std::io::Read;

/// Binary format "tokenizer", which is trivial; the tokens are just bytes.
#[derive(Debug)]
pub struct Tokenizer<R> {
    inner: R,
    location: Location,
}

impl<R> Tokenizer<R> {
    pub fn new(r: R) -> Self {
        Tokenizer {
            inner: r,
            location: Location::default(),
        }
    }

    pub fn location(&self) -> Location {
        self.location
    }
}

impl<R: Read> Read for Tokenizer<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let cnt = self.inner.read(buf)?;
        self.location.advanceby(cnt);
        Ok(cnt)
    }
}
