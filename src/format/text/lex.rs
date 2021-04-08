use std::{io::Read, iter};
use std::iter::Iterator;

use crate::error::{Result, ResultFrom};

fn next_id(r: &mut R) -> Result<Token> {
}

fn tokenize<R : Read>(r: &mut R) -> impl Iterator<Item=Result<Token>> + '_ {
    std::iter::from_fn(move || {
        let next_start = r.bytes().next();
        match next_start {
            None => return None,
            Some(r) => match r {
                Err(e) => return Some(Err(e)),
                Ok(c) => match c as char {
                    '$' => Ok(next_id(r))
                }
            }
        }

    })
}


enum Token {
    Keyword(String),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    String(String),
    Id(String),
    Open,
    Close,
    Reserved(String)
}
