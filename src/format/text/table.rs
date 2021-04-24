use std::io::Read;
use super::{Expr, Parser, elem::ElemList};
use crate::{error::Result, types::TableType};

#[derive(Debug)]
#[allow(dead_code)]
pub enum TableElems {
    Elem(ElemList),
    Expr(Vec<Expr>),
}

#[derive(Debug)]
#[allow(dead_code)]
// Table may either be an import, or declaring a new table,
// in which case the contents may include initializer element segments.
pub enum TableContents {
    Inline{elems: Option<TableElems>},
    Import(String),
}

#[derive(Debug)]
// table :: = (table id? <tabletype>)
// Abbreviations:
// inline imports/exports
// inline elem
pub struct TableField {
    id: Option<String>,
    exports: Vec<String>,
    tabletype: TableType,
    contents: TableContents
}

impl<R: Read> Parser<R> {
    pub fn parse_table_section(&mut self) -> Result<Option<TableField>> {
        self.consume_expression()?; 
        Ok(None)
    }
}
