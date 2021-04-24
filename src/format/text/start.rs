use std::io::Read;
use super::{Index, Parser};
use crate::error::Result;

#[derive(Debug)]
// start := (start <funcidx>)
pub struct StartField {
    idx: Index
}

impl<R: Read> Parser<R> {
    pub fn parse_start_section(&mut self) -> Result<Option<StartField>> {
        self.consume_expression()?; 
        Ok(None)
    }
}
