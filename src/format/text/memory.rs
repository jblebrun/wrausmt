use std::io::Read;
use super::Parser;
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
    pub fn parse_memory_section(&mut self) -> Result<Option<MemoryField>> {
        self.consume_expression()?; 
        Ok(None)
    }
}
