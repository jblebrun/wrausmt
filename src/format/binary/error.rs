use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    Error(String),
    Wrapped(String, Box<dyn Error>)
}

pub type Result<T> = std::result::Result<T, ParseError>;

impl ParseError {
    pub fn new(msg: String) -> ParseError {
        ParseError::Error(msg)
    }
}

impl From<&str> for ParseError {
    fn from(msg: &str) -> ParseError {
        ParseError::new(msg.to_string())
    }
}

pub trait ParseErrorFrom {
    fn wrap(self, msg: &str) -> ParseError;
}

pub trait ResultFrom<T, E> {
    fn wrap(self, msg: &str) -> Result<T>;
}

impl <E : Error + Sized + 'static> ParseErrorFrom for E {
    fn wrap(self, msg: &str) ->  ParseError {
        ParseError::Wrapped(msg.to_string(), Box::new(self))
    }
}

impl <T, E : ParseErrorFrom> ResultFrom<T, E>  for std::result::Result<T,E> {
    fn wrap(self, msg: &str)-> Result<T> {
        self.map_err(|e| e.wrap(msg))
    }
}

impl fmt::Display for ParseError {
    fn fmt<'l>(&'l self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            ParseError::Error(msg) => write!(f, "  {}", msg),
            ParseError::Wrapped(msg, src) => write!(f, "  {}\n{}", msg, src)
        }
    }
}

impl Error for ParseError { }

