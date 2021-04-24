use std::io::Read;
use super::Parser;
use super::Index;
use super::Expr;
use crate::error::Result;

#[derive(Debug)]
pub struct DataInit {
    memidx: Index,
    offset: Expr
}

#[derive(Debug)]
// data := (data id? <datastring>)
//       | (data id? <memuse> (offset <expr>) <datastring>)
// datastring := bytestring
// memuse := (memory <memidx>)
pub struct DataField {
    id: Option<String>,
    data: Vec<u8>,
    init: Option<DataInit>
}

impl<R: Read> Parser<R> {
    pub fn parse_data_section(&mut self) -> Result<Option<DataField>> {
        self.consume_expression()?; 
        Ok(None)
    }
}
