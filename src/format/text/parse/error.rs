use crate::format::text::{lex::error::LexError, resolve::ResolveError};


#[derive(Debug)]
pub enum ParseError {
    Eof,
    Tokenizer(LexError),
    UnexpectedToken(String),
    ResolveError(ResolveError),
    Incomplete
}

impl ParseError {
    pub fn unexpected(expected: &str) -> ParseError {
        ParseError::UnexpectedToken(expected.into())
    }
}

impl std::error::Error for ParseError {
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
    
}

pub type Result<T> = std::result::Result<T, ParseError>;

impl From<ResolveError> for ParseError {
    fn from(re: ResolveError) -> Self {
        ParseError::ResolveError(re)
    }
}
