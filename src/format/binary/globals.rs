use crate::{
    error::{Result, ResultFrom},
    module::Global,
};
use super::{
    code::ReadCode, 
    values::ReadWasmValues
};

/// Read the tables section of a binary module from a std::io::Read.
pub trait ReadGlobals : ReadWasmValues + ReadCode {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_globals_section(&mut self) -> Result<Box<[Global]>>{
        let items = self.read_u32_leb_128().wrap("parsing item count")?;
        (0..items).map(|_| {
            Ok(
                Global {
                    typ: self.read_global_type()?,
                    init: self.read_expr()?
                }
            )
        }).collect()
    }
}

impl <I:ReadWasmValues + ReadCode> ReadGlobals for I {}
