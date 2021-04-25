use std::io::Read;
use super::{Expr, Field, Index, Parser};
use crate::{error::Result, types::RefType};

#[derive(Debug, PartialEq)]
pub struct TableUse {
    tableidx: Index
}

#[derive(Debug, PartialEq)]
pub struct TablePosition {
    tableuse: TableUse,
    offset: Expr
}

#[derive(Debug, PartialEq)]
pub struct ElemList {
    reftype: RefType,
    items: Vec<Expr>
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum ModeEntry {
    Passive,
    Active(TablePosition),
    Declarative
}

// elem := (elem <id>? <elemlist>)
//       | (elem <id>? <tableuse> (offset <expr>) <elemlist>)
//       | (elem <id>? declare <elemlist>)
#[derive(Debug, PartialEq)]
pub struct ElemField {
    id: Option<String>,
    mode: ModeEntry,
    elemlist: ElemList,
}

impl<R: Read> Parser<R> {
    pub fn parse_elem_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("elem")? {
            return Ok(None)
        }
        self.consume_expression()?; 
        Ok(Some(Field::Elem(ElemField {
            id: None,
            mode: ModeEntry::Passive,
            elemlist: ElemList {
                reftype: RefType::Func,
                items: vec![]
            }
        })))
    }
}
