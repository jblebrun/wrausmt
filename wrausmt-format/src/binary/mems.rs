use {
    super::{error::Result, BinaryParser, ParserReader},
    crate::{binary::read_with_location::Locate, pctx},
    wrausmt_runtime::syntax::MemoryField,
};

/// Read the tables section of a binary module from a std::io::Read.
impl<R: ParserReader> BinaryParser<R> {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    pub(in crate::binary) fn read_mems_section(&mut self) -> Result<Vec<MemoryField>> {
        pctx!(self, "read mems section");
        self.read_vec(|_, s| s.read_memory_field())
    }

    fn read_memory_field(&mut self) -> Result<MemoryField> {
        pctx!(self, "read memory field");
        let location = self.location();
        Ok(MemoryField {
            id: None,
            memtype: self.read_memory_type()?,
            exports: vec![],
            location,
        })
    }
}
