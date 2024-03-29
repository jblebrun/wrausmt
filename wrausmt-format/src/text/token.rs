use {
    super::string::WasmString,
    wrausmt_runtime::syntax::{location::Location, Id},
};

/// A [Token] along with context about its location in the source file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FileToken {
    pub token:    Token,
    pub location: Location,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Sign {
    Unspecified,
    Negative,
    Positive,
}

impl Sign {
    pub fn char(&self) -> &str {
        match self {
            Self::Unspecified => "",
            Self::Negative => "-",
            Self::Positive => "+",
        }
    }
}

impl<IC: Into<char>> From<IC> for Sign {
    fn from(ch: IC) -> Sign {
        match ch.into() {
            '+' => Sign::Positive,
            '-' => Sign::Negative,
            _ => Sign::Unspecified,
        }
    }
}

/// An enum of all of the possible lexical tokens that can occur in a
/// WebAssembly text file.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Token {
    #[default]
    Start,
    Keyword(Id),
    Reserved(String),
    Number(NumToken),
    String(WasmString),
    Id(Id),
    Open,
    Close,
    Eof,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Base {
    Dec,
    Hex,
}

impl Base {
    pub fn radix(&self) -> u32 {
        match self {
            Self::Dec => 10,
            Self::Hex => 16,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NumToken {
    NaN(Sign),
    NaNx(Sign, String),
    Inf(Sign),
    Integer(Sign, Base, String),
    Float(Sign, Base, String, String, String),
}
