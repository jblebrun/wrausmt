use std::io::Read;
use super::{Expr, Field, Parser, elem::ElemList};
use crate::{error::Result, types::{Limits, RefType, TableType}};

#[derive(Debug, PartialEq)]
pub enum TableElems {
    Elem(ElemList),
    Expr(Vec<Expr>),
}

#[derive(Debug, PartialEq)]
// Table may either be an import, or declaring a new table,
// in which case the contents may include initializer element segments.
pub enum TableContents {
    Inline{elems: Option<TableElems>},
    Import(String),
}

#[derive(Debug, PartialEq)]
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
    pub fn parse_table_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("table")? {
            return Ok(None)
        }
        self.consume_expression()?; 
        Ok(Some(Field::Table(TableField {
            id: None,
            exports: vec![],
            tabletype: TableType { limits: Limits::default(), reftype: RefType::Func },
            contents: TableContents::Inline{ elems: None}
        })))
    }
}
