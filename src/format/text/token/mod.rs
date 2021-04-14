use crate::error;
use crate::error::Result;

#[derive(Debug, PartialEq)]
pub enum Token {
    Start,
    Whitespace,
    LineComment,
    BlockComment,
    Keyword(String),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    String(String),
    Id(String),
    Open,
    Close,
    Inf,
    NaN,
    NaNx(u32),
    Eof
}

impl Token {

    pub fn ignorable(&self) -> bool {
       matches!(self, Token::Start | Token::Whitespace | Token::LineComment | Token::BlockComment)
    }

    pub fn is_keyword<S : Into<String> + PartialEq<String>>(&self, to_match: S) -> bool {
        matches!(self, Token::Keyword(s) if to_match == s.into())
    }

    pub fn try_keyword(&self) -> Option<&String> {
        match &self {
            Token::Keyword(s)  => Some(s),
            _ => None
        }
    }

    pub fn expect_keyword(&self) -> Result<&String> {
        self.try_keyword()
            .ok_or_else(|| error!("expected keyword, got {:?}", self))
    }
}
