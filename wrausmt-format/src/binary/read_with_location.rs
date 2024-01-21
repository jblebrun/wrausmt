use {std::io::Read, wrausmt_runtime::syntax::location::Location};

#[derive(Debug)]
pub struct ReadWithLocation<R> {
    inner:    R,
    location: Location,
}

pub trait Locate {
    fn location(&self) -> Location;
}

impl<R> ReadWithLocation<R> {
    pub fn new(r: R) -> Self {
        ReadWithLocation {
            inner:    r,
            location: Location::default(),
        }
    }

    pub fn location(&self) -> Location {
        self.location
    }
}

impl<T: Read> Read for ReadWithLocation<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let cnt = self.inner.read(buf)?;
        self.location.advanceby(cnt);
        Ok(cnt)
    }
}

impl<T: Read> Locate for ReadWithLocation<T> {
    fn location(&self) -> Location {
        self.location()
    }
}
