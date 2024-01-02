use {
    super::{error::Result, BinaryParser},
    crate::pctx,
    std::io::Read,
    wrausmt_runtime::syntax::{GlobalField, Resolved},
};

/// Read the tables section of a binary module from a std::io::Read.
impl<R: Read> BinaryParser<R> {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    pub(in crate::binary) fn read_globals_section(&mut self) -> Result<Vec<GlobalField<Resolved>>> {
        pctx!(self, "read globals section");
        self.read_vec(|_, s| s.read_global_field())
    }

    fn read_global_field(&mut self) -> Result<GlobalField<Resolved>> {
        pctx!(self, "read global field");
        Ok(GlobalField {
            id:         None,
            exports:    vec![],
            globaltype: self.read_global_type()?,
            init:       self.read_expr()?,
        })
    }
}
