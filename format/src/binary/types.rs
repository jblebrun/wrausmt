use {
    super::{
        error::{Result, WithContext},
        values::ReadWasmValues,
    },
    wrausmt::syntax::{FunctionType, TypeField},
};

pub trait ReadTypes: ReadWasmValues {
    fn read_types_section(&mut self) -> Result<Vec<TypeField>> {
        self.read_vec(|_, s| {
            s.read_specific_byte(0x60).ctx("checking type byte")?;
            Ok(TypeField {
                id:           None,
                functiontype: FunctionType {
                    params:  s.read_fparam().ctx("parsing params")?,
                    results: s.read_fresult().ctx("parsing result")?,
                },
            })
        })
    }
}

impl<I: ReadWasmValues> ReadTypes for I {}
