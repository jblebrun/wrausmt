use {
    super::{error::Result, BinaryParser},
    std::io::Read,
    wrausmt_runtime::syntax::MemoryField,
};

/// Read the tables section of a binary module from a std::io::Read.
impl<R: Read> BinaryParser<R> {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    pub(in crate::binary) fn read_mems_section(&mut self) -> Result<Vec<MemoryField>> {
        self.read_vec(|_, s| s.read_memory_field())
    }

    fn read_memory_field(&mut self) -> Result<MemoryField> {
        Ok(MemoryField {
            id:      None,
            memtype: self.read_memory_type()?,
            exports: vec![],
        })
    }
}
