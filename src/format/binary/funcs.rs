use crate::error::{Result, ResultFrom};
use crate::module::index;

use super::values::ReadWasmValues;

/// Read the funcs section of a binary module from a std::io::Read.
pub trait ReadFuncs: ReadWasmValues {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_funcs_section(&mut self) -> Result<Box<[index::Type]>> {
        self.read_vec(|_, s| s.read_u32_leb_128().wrap("parsing func"))
    }
}

impl<I: ReadWasmValues> ReadFuncs for I {}
