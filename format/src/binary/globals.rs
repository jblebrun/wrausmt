use {
    super::{code::ReadCode, error::Result, values::ReadWasmValues},
    wrausmt::syntax::{GlobalField, Resolved},
};

/// Read the tables section of a binary module from a std::io::Read.
pub trait ReadGlobals: ReadWasmValues + ReadCode {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_globals_section(&mut self) -> Result<Vec<GlobalField<Resolved>>> {
        self.read_vec(|_, s| s.read_global_field())
    }

    fn read_global_field(&mut self) -> Result<GlobalField<Resolved>> {
        Ok(GlobalField {
            id:         None,
            exports:    vec![],
            globaltype: self.read_global_type()?,
            init:       self.read_expr()?,
        })
    }
}

impl<I: ReadWasmValues + ReadCode> ReadGlobals for I {}
