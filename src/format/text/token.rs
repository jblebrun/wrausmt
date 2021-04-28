use crate::error;
use crate::error::Result;
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

impl Token {
    /// Returns true if the token is ignorable (whitespace, start, or comment) by the parser.
    pub fn ignorable(&self) -> bool {
       matches!(self, Token::Start | Token::Whitespace | Token::LineComment | Token::BlockComment)
    }

    /// Returns true if the token is a [Token::Keyword] token containing the provided keyword.
    pub fn is_keyword<S : Into<String> + PartialEq<String>>(&self, to_match: S) -> bool {
        matches!(self, Token::Keyword(s) if to_match == s.into())
    }

    /// If the [Token] is a [Token::Keyword], this method returns a reference to the contained
    /// [String], else [None].
    pub fn try_keyword(&self) -> Option<&String> {
        match &self {
            Token::Keyword(s)  => Some(s),
            _ => None
        }
    }

    /// If the [Token] is a [Token::Keyword], this method returns a reference to the contained
    /// [String], otherwise it returns an error result.
    pub fn expect_keyword(&self) -> Result<&String> {
        self.try_keyword()
            .ok_or_else(|| error!("expected keyword, got {:?}", self))
    }
}

impl Location {
    /// Create a new [FileToken] from this [Location] for the provided [Token].
    pub fn token(self, token: Token) -> FileToken {
        FileToken { token, location: self }
    }
}
