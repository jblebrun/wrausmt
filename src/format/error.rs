use crate::error::Error;
use crate::format::Location;

#[derive(Debug)]
pub struct ParseError {
    cause: Error,
    location: Location,
}

impl ParseError {
    pub fn new(cause: Error, location: Location) -> Self {
        ParseError { cause, location }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt<'l>(&'l self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error parsing binary module at {:?}", self.location)
    }
}

impl std::error::Error for ParseError {}
