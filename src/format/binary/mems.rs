use super::error::Result;
use super::{leb128::ReadLeb128, values::ReadWasmValues};
use crate::syntax::MemoryField;

/// Read the tables section of a binary module from a std::io::Read.
pub trait ReadMems: ReadWasmValues {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_mems_section(&mut self) -> Result<Vec<MemoryField>> {
        self.read_vec(|_, s| s.read_memory_field())
    }

    fn read_memory_field(&mut self) -> Result<MemoryField> {
        Ok(MemoryField {
            id: None,
            memtype: self.read_memory_type()?,
            exports: vec![],
        })
    }
}

impl<I: ReadLeb128> ReadMems for I {}
