use super::values::ReadWasmValues;
use crate::{
    error::{Result, ResultFrom},
    syntax::{Resolved, TableField},
};

/// Read the tables section of a binary module from a std::io::Read.
pub trait ReadTables: ReadWasmValues {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_tables_section(&mut self) -> Result<Vec<TableField<Resolved>>> {
        self.read_vec(|_, s| s.read_table_field().wrap("read table type"))
    }

    fn read_table_field(&mut self) -> Result<TableField<Resolved>> {
        Ok(TableField {
            id: None,
            tabletype: self.read_table_type()?,
            exports: vec![],
            elems: None,
        })
    }
}

impl<I: ReadWasmValues> ReadTables for I {}
