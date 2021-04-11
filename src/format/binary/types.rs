use super::values::ReadWasmValues;
use crate::types::FunctionType;
use crate::error::{Result, ResultFrom};

pub trait ReadTypes : ReadWasmValues {
    fn read_types_section(&mut self) -> Result<Box<[FunctionType]>> { 
        let items = self.read_u32_leb_128().wrap("parsing item count")?;

        (0..items).map(|_| {
            self.read_specific_byte(0x60).wrap("checking type byte")?;
            Ok(FunctionType {
                params: self.read_result_type().wrap("parsing params")?,
                result: self.read_result_type().wrap("parsing result")?
            })
        }).collect()
    }
}

impl <I:ReadWasmValues> ReadTypes for I {}
