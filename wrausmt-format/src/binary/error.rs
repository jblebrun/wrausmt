use {
    super::{leb128::LEB128Error, read_with_location::Locate, BinaryParser, ParserReader},
    std::{io::Read, string::FromUtf8Error},
    wrausmt_runtime::syntax::{location::Location, Opcode},
};

#[derive(Debug)]
pub enum BinaryParseErrorKind {
    IncorrectMagic([u8; 4]),
    IncorrectVersion([u8; 4]),
    IOError(std::io::Error),
    LEB128Error(LEB128Error),
    Utf8Error(FromUtf8Error),
    DataCountMismatch,
    DataCountMissing,
    FuncSizeMismatch,
    InvalidOpcode(Opcode),
    InvalidSecondaryOpcode(u32),
    InvalidBoolValue(u8),
    InvalidElemKind(u8),
    InvalidValueType(u8),
    InvalidExportType(u8),
    InvalidFuncType(u8),
    InvalidBlockType(i64),
    ZeroByteExpected,
    CodeTooShort,
    CodeTooLong,
    SectionTooShort,
    SectionTooLong,
    MalformedRefType(u8),
    MalformedSectionId(u8),
    MalformedImportKind(u8),
    UnxpectedEndOfSectionOrFunction,
    UnexpectedContentAfterEnd,
    UnexpectedEnd,
    TooManyLocals,
}

#[derive(Debug)]
pub struct BinaryParseError {
    pub kind: BinaryParseErrorKind,
    msgs:     Vec<String>,
    location: Location,
}

impl BinaryParseError {
    pub fn new(kind: BinaryParseErrorKind, msgs: Vec<String>, location: Location) -> Self {
        Self {
            kind,
            msgs,
            location,
        }
    }

    pub fn with_location(self, location: Location) -> Self {
        Self {
            kind: self.kind,
            msgs: self.msgs,
            location,
        }
    }
}

impl From<std::io::Error> for BinaryParseErrorKind {
    fn from(e: std::io::Error) -> BinaryParseErrorKind {
        BinaryParseErrorKind::IOError(e)
    }
}

impl From<LEB128Error> for BinaryParseErrorKind {
    fn from(e: LEB128Error) -> BinaryParseErrorKind {
        match e {
            LEB128Error::IOError(io) => BinaryParseErrorKind::IOError(io),
            _ => BinaryParseErrorKind::LEB128Error(e),
        }
    }
}

impl From<FromUtf8Error> for BinaryParseErrorKind {
    fn from(e: FromUtf8Error) -> BinaryParseErrorKind {
        BinaryParseErrorKind::Utf8Error(e)
    }
}

impl std::fmt::Display for BinaryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}: (at {:?}) [{:?}]",
            self.kind, self.location, self.msgs
        )
    }
}

pub trait ParseError<T: Read + Locate> {
    fn err(self, parser: BinaryParser<T>) -> BinaryParseError;
}

pub trait ParseResult<T: ParserReader, RT> {
    fn result(self, parser: &mut BinaryParser<T>) -> Result<RT>;
}

impl<T: ParserReader, E: Into<BinaryParseErrorKind>> ParseError<T> for E {
    fn err(self, parser: BinaryParser<T>) -> BinaryParseError {
        parser.err(self.into())
    }
}

impl<T: ParserReader, RT, E: Into<BinaryParseErrorKind>> ParseResult<T, RT>
    for std::result::Result<RT, E>
{
    fn result(self, parser: &mut BinaryParser<T>) -> Result<RT> {
        self.map_err(|e| parser.err(e.into()))
    }
}

impl std::error::Error for BinaryParseError {}

/// Most functions internally work with BinaryParseErrorKind as a type.
pub(in crate::binary) type Result<T> = std::result::Result<T, BinaryParseError>;

pub trait EofAsKind {
    fn eof_as_kind(self, kind: BinaryParseErrorKind) -> Self;
}

impl<T> EofAsKind for Result<T> {
    fn eof_as_kind(self, kind: BinaryParseErrorKind) -> Self {
        self.map_err(|mut se| match &se {
            BinaryParseError {
                kind: BinaryParseErrorKind::IOError(e),
                ..
            } if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                se.kind = kind;
                se
            }
            _ => se,
        })
    }
}
