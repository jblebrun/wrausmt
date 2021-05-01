use crate::format::Location;

/// A [Token] along with context about its location in the source file.
#[derive(Debug, Default, PartialEq)]
pub struct FileToken {
    pub token: Token,
    pub location: Location,
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Sign {
    Unspecified,
    Negative,
    Positive,
}

impl <IC:Into<char>> From<IC> for Sign {
    fn from(ch: IC) -> Sign {
        match ch.into() {
            '+' => Sign::Positive,
            '-' => Sign::Negative,
            _ => Sign::Unspecified 
        }
    }
}

/// An enum of all of the possible lexical tokens that can occur in a WebAssembly text file.
#[derive(Debug, PartialEq)]
pub enum Token {
    Start,
    Whitespace,
    LineComment,
    BlockComment,
    Keyword(String),
    Reserved(String),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    String(String),
    Id(String),
    Open,
    Close,
    Inf(Sign),
    NaN(Sign),
    NaNx(Sign, u32),
    Eof
}

impl Default for Token {
    /// Returns a default token of [Token::Start].
    fn default() -> Token { Token::Start }
}

impl Location {
    /// Create a new [FileToken] from this [Location] for the provided [Token].
    pub fn token(self, token: Token) -> FileToken {
        FileToken { token, location: self }
    }
}
