use std::io::Read;
use super::Parser;
use super::Index;
use super::Field;
use super::Expr;
use crate::error::Result;

#[derive(Debug, PartialEq)]
pub struct DataInit {
    memidx: Index,
    offset: Expr
}

// data := (data id? <datastring>)
//       | (data id? <memuse> (offset <expr>) <datastring>)
// datastring := bytestring
// memuse := (memory <memidx>)
#[derive(Debug, PartialEq)]
pub struct DataField {
    id: Option<String>,
    data: Vec<u8>,
    init: Option<DataInit>
}

impl<R: Read> Parser<R> {
    pub fn parse_data_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("data")? {
            return Ok(None)
        }
        self.consume_expression()?; 
        Ok(Some(Field::Data(DataField {
            id: None,
            data: vec![],
            init: None
        })))
    }
}
