use {super::leb128::LEB128Error, std::string::FromUtf8Error, wrausmt_runtime::syntax::Opcode};

#[derive(Debug)]
pub enum BinaryParseErrorKind {
    WithContext(Vec<String>, Box<BinaryParseErrorKind>),
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

#[derive(Debug)]
pub struct BinaryParseError {
    kind: BinaryParseErrorKind,
    msgs: Vec<String>,
}

impl BinaryParseError {
    pub fn new(kind: BinaryParseErrorKind) -> Self {
        Self {
            kind,
            msgs: Vec::new(),
        }
    }
}
pub trait WithContext<T> {
    fn ctx(self, msg: impl Into<String>) -> T;
}

impl WithContext<BinaryParseError> for BinaryParseErrorKind {
    fn ctx(self, msg: impl Into<String>) -> BinaryParseError {
        BinaryParseError {
            kind: self,
            msgs: vec![msg.into()],
        }
    }
}
impl WithContext<BinaryParseError> for BinaryParseError {
    fn ctx(mut self, msg: impl Into<String>) -> BinaryParseError {
        self.msgs.push(msg.into());
        self
    }
}
impl<T> WithContext<Result<T>> for Result<T> {
    fn ctx(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| e.ctx(msg))
    }
}

impl<T, E: Into<BinaryParseErrorKind>> WithContext<Result<T>> for std::result::Result<T, E> {
    fn ctx(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| e.into().ctx(msg))
    }
}

impl From<BinaryParseErrorKind> for BinaryParseError {
    fn from(value: BinaryParseErrorKind) -> Self {
        BinaryParseError::new(value)
    }
}
impl From<std::io::Error> for BinaryParseErrorKind {
    fn from(e: std::io::Error) -> BinaryParseErrorKind {
        BinaryParseErrorKind::IOError(e)
    }
}

impl From<LEB128Error> for BinaryParseErrorKind {
    fn from(e: LEB128Error) -> BinaryParseErrorKind {
        BinaryParseErrorKind::LEB128Error(e)
    }
}

impl From<LEB128Error> for BinaryParseError {
    fn from(e: LEB128Error) -> BinaryParseError {
        BinaryParseErrorKind::LEB128Error(e).into()
    }
}

impl From<FromUtf8Error> for BinaryParseErrorKind {
    fn from(e: FromUtf8Error) -> BinaryParseErrorKind {
        BinaryParseErrorKind::Utf8Error(e)
    }
}

impl std::fmt::Display for BinaryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: [{:?}]", self.kind, self.msgs)
    }
}

impl std::error::Error for BinaryParseError {}

pub type Result<T> = std::result::Result<T, BinaryParseError>;
