use std::io::Read;
use super::{Field, Index, Parser};
use crate::error::Result;

// start := (start <funcidx>)
#[derive(Debug, PartialEq)]
pub struct StartField {
    idx: Index
}

impl<R: Read> Parser<R> {
    pub fn parse_start_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("start")? {
            return Ok(None)
        } 
        self.consume_expression()?; 
        Ok(Some(Field::Start(StartField { idx:Index::Numeric(42) })))
    }
}
