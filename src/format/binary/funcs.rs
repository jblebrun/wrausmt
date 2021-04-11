use crate::error::{Result, ResultFrom};
use crate::module::index;

use super::leb128::ReadLeb128;

/// Read the funcs section of a binary module from a std::io::Read.
pub trait ReadFuncs : ReadLeb128 {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_funcs_section(&mut self) -> Result<Box<[index::Type]>> {
        let items = self.read_u32_leb_128().wrap("parsing item count")?;
        (0..items).map(|_| {
            self.read_u32_leb_128().wrap("parsing func")
        }).collect()
    }
}

impl <I:ReadLeb128> ReadFuncs for I {}
