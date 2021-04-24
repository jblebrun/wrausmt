use std::io::Read;
use super::{Field, Parser, TypeUse};
use crate::{error::Result, types::{GlobalType, MemType, TableType}};

#[derive(Debug, PartialEq)]
pub enum ImportDesc {
    Func(TypeUse),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

impl Default for ImportDesc {
    fn default() -> Self {
        Self::Func(TypeUse::default())
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct ImportField {
    modname: String,
    name: String,
    id: Option<String>,
    desc: ImportDesc
}

impl<R: Read> Parser<R> {
    pub fn parse_import_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("import")? {
            return Ok(None)
        }
        self.consume_expression()?; 
        Ok(Some(Field::Import(ImportField::default())))
    }
}
