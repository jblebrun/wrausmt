use std::io::{Read, Result};

/// CountRead wraps a Read type [T] with state about how many
/// bytes have been consumed so far.
pub struct CountRead<T> {
    inner: T,
    consumed: usize
}

impl <T> CountRead<T> {
    pub fn new(inner: T) -> CountRead<T> {
        CountRead {
            inner,
            consumed: 0
        }
    }

    /// Return the number of bytes consumed by read calls since this instance was created.
    pub fn consumed(&self) -> usize { self.consumed }
}

impl <T : Read> Read for CountRead<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.inner.read(buf) {
            Ok(c) => { self.consumed += c; Ok(c) },
            Err(e) => Err(e)
        }
    }
}
