use {super::string::WasmString, crate::Location, wrausmt_runtime::syntax::Id};

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

#[derive(Clone, Debug, PartialEq)]
pub enum Reserved {
    Id,
    String,
}
/// An enum of all of the possible lexical tokens that can occur in a
/// WebAssembly text file.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Start,
    Whitespace,
    LineComment,
    BlockComment,
    Keyword(Id),
    Reserved(String),
    Number(NumToken),
    String(WasmString),
    Id(Id),
    Open,
    Close,
    Eof,
}

#[derive(Clone, Debug, PartialEq)]
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

impl Default for Token {
    /// Returns a default token of [Token::Start].
    fn default() -> Token {
        Token::Start
    }
}

impl Location {
    /// Create a new [FileToken] from this [Location] for the provided [Token].
    pub fn token(self, token: Token) -> FileToken {
        FileToken {
            token,
            location: self,
        }
    }
}
