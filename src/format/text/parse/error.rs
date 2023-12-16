use std::{
    num::{ParseFloatError, ParseIntError},
    string::FromUtf8Error,
};

use crate::format::text::{lex::error::LexError, resolve::ResolveError, token::FileToken};

#[derive(Debug)]
pub struct ParseErrorContext {
    pub context: Vec<String>,
    pub current: FileToken,
    pub next: FileToken,
}

#[derive(Debug)]
pub enum ParseError {
    WithContext(Box<ParseErrorContext>, Box<ParseError>),
    WithMsg(Vec<String>, Box<ParseError>),
    Eof,
    IoError(std::io::Error),
    LexError(LexError),
    UnexpectedToken(String),
    UnrecognizedInstruction(String),
    ResolveError(ResolveError),
    Utf8Error(FromUtf8Error),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    Incomplete,
}

impl ParseError {
    pub fn unexpected(expected: &str) -> ParseError {
        ParseError::UnexpectedToken(expected.into())
    }
}
pub trait WithMsg<T> {
    fn msg(self, msg: impl Into<String>) -> T;
}

impl WithMsg<ParseError> for ParseError {
    fn msg(mut self, msg: impl Into<String>) -> ParseError {
        match self {
            ParseError::WithMsg(ref mut msgs, _) => {
                let s = msg.into();
                println!("ADD MSG {:?}", s);
                msgs.push(s);
                self
            }
            _ => {
                let s = msg.into();
                println!("NEW MSG {:?}", s);
                ParseError::WithMsg(vec![s], Box::new(self))
            }
        }
    }
}

impl<T> WithMsg<Result<T>> for Result<T> {
    fn msg(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| e.msg(msg))
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

impl From<ParseFloatError> for ParseError {
    fn from(e: ParseFloatError) -> Self {
        ParseError::ParseFloatError(e)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(e: ParseIntError) -> Self {
        ParseError::ParseIntError(e)
    }
}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        ParseError::LexError(e)
    }
}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        ParseError::IoError(e)
    }
}
