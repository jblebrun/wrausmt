use std::fmt;

#[derive(Debug)]
pub enum Error {
    Error(String),
    Wrapped(String, Box<dyn std::error::Error>)
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(msg: String) -> Error {
        Error::Error(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Error {
        Error::new(msg.to_string())
    }
}

pub trait ErrorFrom {
    fn wrap(self, msg: &str) -> Error;
}

pub trait ResultFrom<T, E> {
    fn wrap(self, msg: &str) -> Result<T>;
}

impl <E : std::error::Error + Sized + 'static> ErrorFrom for E {
    fn wrap(self, msg: &str) ->  Error {
        Error::Wrapped(msg.to_string(), Box::new(self))
    }
}

impl <T, E : ErrorFrom> ResultFrom<T, E>  for std::result::Result<T,E> {
    fn wrap(self, msg: &str)-> Result<T> {
        self.map_err(|e| e.wrap(msg))
    }
}

impl fmt::Display for Error {
    fn fmt<'l>(&'l self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Error::Error(msg) => write!(f, "  {}", msg),
            Error::Wrapped(msg, src) => write!(f, "  {}\n{}", msg, src)
        }
    }
}

impl std::error::Error for Error {}

