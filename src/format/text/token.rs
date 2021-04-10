#[derive(Debug, PartialEq)]
pub enum Token {
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
}

