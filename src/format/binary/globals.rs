use super::{code::ReadCode, values::ReadWasmValues};
use crate::{error::Result, error::ResultFrom, module::Global};

/// Read the tables section of a binary module from a std::io::Read.
pub trait ReadGlobals: ReadWasmValues + ReadCode {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_globals_section(&mut self) -> Result<Box<[Global]>> {
        self.read_vec(|_, s| {
            Ok(Global {
                typ: s.read_global_type().wrap("reading global type")?,
                init: s.read_expr().wrap("reading global init")?,
            })
        })
    }
}

impl<I: ReadWasmValues + ReadCode> ReadGlobals for I {}
