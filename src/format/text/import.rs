use std::io::Read;
use super::{Parser, TypeUse};
use crate::{error::Result, types::{GlobalType, MemType, TableType}};

#[derive(Debug)]
#[allow(dead_code)]
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

#[derive(Debug, Default)]
pub struct ImportField {
    modname: String,
    name: String,
    id: Option<String>,
    desc: ImportDesc
}

impl<R: Read> Parser<R> {
    pub fn parse_import_section(&mut self) -> Result<Option<ImportField>> {
        self.consume_expression()?; 
        Ok(Some(ImportField::default()))
    }
}
