use std::{
    num::{ParseFloatError, ParseIntError},
    string::FromUtf8Error,
};

use crate::format::text::{lex::error::LexError, resolve::ResolveError, token::FileToken};

#[derive(Debug, Default)]
pub struct ParseContext {
    pub current: FileToken,
    pub next:    FileToken,
}

#[derive(Debug, Default)]
pub enum ParseErrorKind {
    #[default]
    Unknown,
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

#[derive(Debug, Default)]
pub struct ParseError {
    kind:    ParseErrorKind,
    context: ParseContext,
    msgs:    Vec<String>,
}

impl ParseError {
    pub fn new_nocontext(kind: ParseErrorKind) -> Self {
        Self {
            kind,
            ..Default::default()
        }
    }

    pub fn new(kind: ParseErrorKind, context: ParseContext) -> Self {
        Self {
            kind,
            context,
            msgs: Vec::new(),
        }
    }

    pub fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }

    pub fn context(&self) -> &ParseContext {
        &self.context
    }
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<T> = std::result::Result<T, ParseError>;
pub type KindResult<T> = std::result::Result<T, ParseErrorKind>;

pub trait WithMsg {
    fn msg(self, msg: impl Into<String>) -> Self;
}
impl<T> WithMsg for Result<T> {
    fn msg(mut self, msg: impl Into<String>) -> Self {
        if let Err(e) = &mut self {
            e.msgs.push(msg.into())
        }
        self
    }
}

impl From<ResolveError> for ParseErrorKind {
    fn from(re: ResolveError) -> Self {
        ParseErrorKind::ResolveError(re)
    }
}

impl From<FromUtf8Error> for ParseErrorKind {
    fn from(e: FromUtf8Error) -> Self {
        ParseErrorKind::Utf8Error(e)
    }
}

impl From<ParseFloatError> for ParseErrorKind {
    fn from(e: ParseFloatError) -> Self {
        ParseErrorKind::ParseFloatError(e)
    }
}

impl From<ParseIntError> for ParseErrorKind {
    fn from(e: ParseIntError) -> Self {
        ParseErrorKind::ParseIntError(e)
    }
}

impl From<LexError> for ParseErrorKind {
    fn from(e: LexError) -> Self {
        ParseErrorKind::LexError(e)
    }
}
