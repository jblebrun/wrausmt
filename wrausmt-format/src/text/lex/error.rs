use wrausmt_runtime::syntax::IdError;

#[derive(Debug)]
pub enum LexError {
    WithContext(Vec<String>, Box<LexError>),
    IoError(std::io::Error),
    FromUtf8Error(std::string::FromUtf8Error),
    IdError(IdError),
    Utf8Error(std::str::Utf8Error),
    UnexpectedChar(char),
    InvalidEscape(String),
    UnexpectedEof,
}

pub type Result<T> = std::result::Result<T, LexError>;

pub trait WithContext<T>: Sized {
    fn ctx(self, ctx: &str) -> T;
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for LexError {}

impl From<std::io::Error> for LexError {
    fn from(e: std::io::Error) -> Self {
        LexError::IoError(e)
    }
}

impl From<std::string::FromUtf8Error> for LexError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        LexError::FromUtf8Error(e)
    }
}

impl From<std::str::Utf8Error> for LexError {
    fn from(e: std::str::Utf8Error) -> Self {
        LexError::Utf8Error(e)
    }
}

impl From<IdError> for LexError {
    fn from(value: IdError) -> Self {
        LexError::IdError(value)
    }
}
