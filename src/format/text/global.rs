use std::io::Read;
use super::{Expr, Parser};
use crate::{error::Result, types::{GlobalType, RefType, ValueType}};

#[derive(Debug)]
// global := (global <id>? <globaltype> <expr>)
pub struct GlobalField {
    id: Option<String>,
    globaltype: GlobalType,
    init: Expr
}

impl<R: Read> Parser<R> {
    pub fn parse_global_section(&mut self) -> Result<Option<GlobalField>> {
        self.consume_expression()?; 
        Ok(Some(GlobalField {
            id: None,
            globaltype: GlobalType { mutable: false, valtype: ValueType::Ref(RefType::Func) },
            init: Expr{}
        }))
    }
}
