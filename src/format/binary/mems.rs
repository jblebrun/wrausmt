use super::leb128::ReadLeb128;
use super::values::ReadWasmValues;
use crate::error::Result;
use crate::module::Memory;

/// Read the tables section of a binary module from a std::io::Read.
pub trait ReadMems: ReadWasmValues {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_mems_section(&mut self) -> Result<Box<[Memory]>> {
        self.read_vec(|_, s| s.read_memory_type())
    }
}

impl<I: ReadLeb128> ReadMems for I {}
