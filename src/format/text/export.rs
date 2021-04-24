use std::io::Read;
use super::{Field, Parser, TypeUse};
use crate::{error::Result, types::{GlobalType, MemType, TableType}};

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum ExportDesc {
    Func(TypeUse),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

// export := (export <name> <exportdesc>)
#[derive(Debug, PartialEq)]
pub struct ExportField {
    name: String,
    exportdesc: ExportDesc
}

impl<R: Read> Parser<R> {
    pub fn parse_export_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("export")? {
            return Ok(None)
        }
        self.consume_expression()?; 
        Ok(Some(Field::Export(ExportField {
            name: "name".into(),
            exportdesc: ExportDesc::Func(TypeUse::default())
        })))
    }
}
