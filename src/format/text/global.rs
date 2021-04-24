use super::{Expr, Field, Parser};
use crate::{
    error::Result,
    types::{GlobalType, RefType, ValueType},
};
use std::io::Read;

// global := (global <id>? <globaltype> <expr>)
#[derive(Debug, PartialEq)]
pub struct GlobalField {
    id: Option<String>,
    globaltype: GlobalType,
    init: Expr,
}

impl<R: Read> Parser<R> {
    pub fn parse_global_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("global")? {
            return Ok(None);
        }

        self.consume_expression()?;
        Ok(Some(Field::Global(GlobalField {
            id: None,
            globaltype: GlobalType {
                mutable: false,
                valtype: ValueType::Ref(RefType::Func),
            },
            init: Expr {},
        })))
    }
}
