use std::io::Read;
use super::{FParam, FResult, Field, Parser};
use crate::error::Result;

// type := (type id? <functype>)
// functype := (func <param>* <result>*)
#[derive(Debug, Default)]
pub struct TypeField {
    id: Option<String>,
    params: Vec<FParam>,
    result: Vec<FResult>
}

impl<R: Read> Parser<R> {
    pub fn parse_type_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("type")? {
            return Ok(None)
        }
        self.consume_expression()?; 
        Ok(Some(Field::Type(TypeField::default())))
    }
}
