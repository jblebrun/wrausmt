use {
    crate::text::{lex::error::LexError, resolve::ResolveError, token::FileToken},
    std::{
        num::{ParseFloatError, ParseIntError},
        string::FromUtf8Error,
    },
    wrausmt_runtime::syntax::Id,
};

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
    InvalidAlignment(u32),
    InvalidNaN(u64),
    TooManyLocals,
    Incomplete,
    LabelMismatch(Option<Id>, Option<Id>),
}

#[derive(Debug, Default)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    context:  ParseContext,
    msgs:     Vec<String>,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, context: ParseContext, msgs: Vec<String>) -> Self {
        Self {
            kind,
            context,
            msgs,
        }
    }

    pub fn context(&self) -> &ParseContext {
        &self.context
    }
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?} ({:?})", self.kind, self.context, self.msgs)
    }
}

impl std::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<T> = std::result::Result<T, ParseError>;
pub type KindResult<T> = std::result::Result<T, ParseErrorKind>;

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
