use std::string::FromUtf8Error;

use crate::format::text::{lex::error::LexError, resolve::ResolveError, token::FileToken};

#[derive(Debug)]
pub struct ParseErrorContext {
    pub context: Vec<String>,
    pub current: FileToken,
    pub next: FileToken,
}

#[derive(Debug)]
pub enum ParseError {
    WithContext(ParseErrorContext, Box<ParseError>),
    Eof,
    Tokenizer(LexError),
    UnexpectedToken(String),
    UnrecognizedInstruction(String),
    ResolveError(ResolveError),
    Utf8Error(FromUtf8Error),
    Incomplete,
}

impl ParseError {
    pub fn unexpected(expected: &str) -> ParseError {
        ParseError::UnexpectedToken(expected.into())
    }
}

impl std::error::Error for ParseError {}

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

impl From<FromUtf8Error> for ParseError {
    fn from(e: FromUtf8Error) -> Self {
        ParseError::Utf8Error(e)
    }
}
