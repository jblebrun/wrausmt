use super::values::ReadWasmValues;
use crate::error::{Result, ResultFrom};
use crate::module::Table;

/// Read the tables section of a binary module from a std::io::Read.
pub trait ReadTables : ReadWasmValues {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_tables_section(&mut self) -> Result<Box<[Table]>>{
        let items = self.read_u32_leb_128().wrap("parsing item count")?;
        (0..items).map(|_| {
            self.read_table_type()
        }).collect()
    }
}

impl <I:ReadWasmValues> ReadTables for I {}
