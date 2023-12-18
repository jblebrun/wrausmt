use {
    super::error::{Result, WithContext},
    crate::syntax::{Index, Resolved, TypeIndex},
};

use super::values::ReadWasmValues;

/// Read the funcs section of a binary module from a std::io::Read.
pub trait ReadFuncs: ReadWasmValues {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_funcs_section(&mut self) -> Result<Vec<Index<Resolved, TypeIndex>>> {
        self.read_vec(|_, s| s.read_index_use().ctx("parsing func"))
    }
}

impl<I: ReadWasmValues> ReadFuncs for I {}
