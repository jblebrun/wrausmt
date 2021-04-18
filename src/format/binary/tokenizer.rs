use crate::format::Location;
use std::io::Read;

/// Binary format "tokenizer", which is trivial; the tokens are just bytes.
pub struct Tokenizer<R> {
    inner: R,
    location: Location
}

impl <R> Tokenizer<R> {
    pub fn new(r: R) -> Self {
        Tokenizer { inner: r, location: Location::default() }
    }

    pub fn location(&self) -> Location {
        self.location
    }
}

impl<R: Read> Read for Tokenizer<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.inner.read(buf) {
            Ok(c) => {
                self.location.nextchar();
                Ok(c)
            }
            Err(e) => Err(e),
        }
    }
}

