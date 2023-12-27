#[derive(Debug)]
pub enum LexError {
    WithContext(Vec<String>, Box<LexError>),
    IoError(std::io::Error),
    Utf8Error(std::string::FromUtf8Error),
    UnexpectedChar(char),
    InvalidEscape(String),
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

impl WithContext<LexError> for LexError {
    fn ctx(self, ctx: &str) -> Self {
        match self {
            LexError::WithContext(mut ctxs, e) => {
                ctxs.push(ctx.to_owned());
                LexError::WithContext(ctxs, e)
            }
            e => LexError::WithContext(vec![ctx.to_owned()], Box::new(e)),
        }
    }
}

impl<T, E: Into<LexError>> WithContext<Result<T>> for std::result::Result<T, E> {
    fn ctx(self, ctx: &str) -> Result<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into().ctx(ctx)),
        }
    }
}

impl From<std::io::Error> for LexError {
    fn from(e: std::io::Error) -> Self {
        LexError::IoError(e)
    }
}

impl From<std::string::FromUtf8Error> for LexError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        LexError::Utf8Error(e)
    }
}
