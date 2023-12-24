use {super::leb128::LEB128Error, crate::syntax::Opcode, std::string::FromUtf8Error};

#[derive(Debug)]
pub enum BinaryParseError {
    WithContext(Vec<String>, Box<BinaryParseError>),
    IOError(std::io::Error),
    Unexpected {
        got:    Box<[u8]>,
        expect: Box<[u8]>,
    },
    LEB128Error(LEB128Error),
    Utf8Error(FromUtf8Error),
    DataCountMismatch,
    FuncSizeMismatch,
    InvalidOpcode(Opcode),
    InvalidSecondaryOpcode(u32),
    InvalidBoolValue(u8),
    InvalidElemKind(u8),
    InvalidValueType(u8),
    InvalidRefType(u8),
    InvalidExportType(u8),
    InvalidImportType(u8),
    ExtraSectionBytes(u64),
    TooManyLocals,
}

pub trait WithContext<T> {
    fn ctx(self, msg: impl Into<String>) -> T;
}

impl WithContext<BinaryParseError> for BinaryParseError {
    fn ctx(self, msg: impl Into<String>) -> BinaryParseError {
        BinaryParseError::WithContext(vec![msg.into()], Box::new(self))
    }
}

impl<T, E: Into<BinaryParseError>> WithContext<Result<T>> for std::result::Result<T, E> {
    fn ctx(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| e.into().ctx(msg))
    }
}

impl WithContext<BinaryParseError> for std::io::Error {
    fn ctx(self, msg: impl Into<String>) -> BinaryParseError {
        BinaryParseError::WithContext(vec![msg.into()], Box::new(BinaryParseError::IOError(self)))
    }
}
impl From<std::io::Error> for BinaryParseError {
    fn from(e: std::io::Error) -> BinaryParseError {
        BinaryParseError::IOError(e)
    }
}

impl From<LEB128Error> for BinaryParseError {
    fn from(e: LEB128Error) -> BinaryParseError {
        BinaryParseError::LEB128Error(e)
    }
}

impl From<FromUtf8Error> for BinaryParseError {
    fn from(e: FromUtf8Error) -> BinaryParseError {
        BinaryParseError::Utf8Error(e)
    }
}

impl std::fmt::Display for BinaryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for BinaryParseError {}

pub type Result<T> = std::result::Result<T, BinaryParseError>;
