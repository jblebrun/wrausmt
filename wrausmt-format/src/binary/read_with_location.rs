use std::io::Read;

#[derive(Debug)]
pub struct ReadWithLocation<R> {
    inner:    R,
    location: usize,
}

impl<R> ReadWithLocation<R> {
    pub fn new(r: R) -> Self {
        ReadWithLocation {
            inner:    r,
            location: 0,
        }
    }

    pub fn location(&self) -> usize {
        self.location
    }
}

impl<T: Read> Read for ReadWithLocation<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let cnt = self.inner.read(buf)?;
        self.location += cnt;
        Ok(cnt)
    }
}
