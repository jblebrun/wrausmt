use super::values::ReadWasmValues;
use crate::error::{Result, ResultFrom};
use crate::types::FunctionType;

pub trait ReadTypes: ReadWasmValues {
    fn read_types_section(&mut self) -> Result<Box<[FunctionType]>> {
        self.read_vec(|_, s| {
            s.read_specific_byte(0x60).wrap("checking type byte")?;
            Ok(FunctionType {
                params: s.read_result_type().wrap("parsing params")?,
                result: s.read_result_type().wrap("parsing result")?,
            })
        })
    }
}

impl<I: ReadWasmValues> ReadTypes for I {}
