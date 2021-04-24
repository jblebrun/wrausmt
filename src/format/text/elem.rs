use std::io::Read;
use super::{Expr, Index, Parser};
use crate::{error::Result, types::RefType};

#[derive(Debug)]
pub struct TableUse {
    tableidx: Index
}

#[derive(Debug)]
pub struct TablePosition {
    tableuse: TableUse,
    offset: Expr
}

#[derive(Debug)]
pub struct ElemList {
    reftype: RefType,
    items: Vec<Expr>
}

#[derive(Debug)]
#[allow(dead_code)]
enum ModeEntry {
    Passive,
    Active(TablePosition),
    Declarative
}

// elem := (elem <id>? <elemlist>)
//       | (elem <id>? <tableuse> (offset <expr>) <elemlist>)
//       | (elem <id>? declare <elemlist>)
#[derive(Debug)]
pub struct ElemField {
    id: Option<String>,
    mode: ModeEntry,
    elemlist: ElemList,
}

impl<R: Read> Parser<R> {
    pub fn parse_elem_section(&mut self) -> Result<Option<ElemField>> {
        self.consume_expression()?; 
        Ok(None)
    }
}
