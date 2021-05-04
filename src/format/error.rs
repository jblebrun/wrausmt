use crate::error::Error;
use crate::format::Location;
use crate::module::Module;

#[derive(Debug)]
pub struct ParseError {
    cause: Error,
    location: Location,
    module: Module,
}

impl ParseError {
    pub fn new(cause: Error, location: Location, module: Module) -> Self {
        ParseError {
            cause,
            location,
            module,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt<'l>(&'l self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error parsing binary module at {:?}\nContents so far:{:?}",
            self.location, self.module
        )
    }
}

impl std::error::Error for ParseError {}
