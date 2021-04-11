use crate::{
    error::Result,
    module::Start,
};
use super::{
    values::ReadWasmValues
};

/// Read the tables section of a binary module from a std::io::Read.
pub trait ReadStart: ReadWasmValues {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_start_section(&mut self) -> Result<Option<Start>>{
        Ok(Some(self.read_u32_leb_128()?))
    }
}

impl <I:ReadWasmValues> ReadStart for I {}
