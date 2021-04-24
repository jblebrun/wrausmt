use std::io::Read;
use super::{Field, Parser};
use crate::{error::Result, types::MemType};
    
#[derive(Debug)]
#[allow(dead_code)]
pub enum MemoryContents {
    // standard
    Inline(MemType),
    // inline init
    Initialized(Vec<u8>),
    // inline import
    Import(String)
}

#[derive(Debug)]
// memory := (memory id? <memtype>)
//
// Abbreviations:
// Inline import/export
// Inline data segments
pub struct MemoryField {
    id: Option<String>,
    exports: Vec<String>,
    contents: MemoryContents,
}

impl<R: Read> Parser<R> {
    pub fn parse_memory_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("memory")? {
            return Ok(None)
        }
        self.consume_expression()?; 
        Ok(Some(Field::Memory(MemoryField {
            id: None,
            exports: vec![],
            contents: MemoryContents::Import("foo".into())
        })))
    }
}
