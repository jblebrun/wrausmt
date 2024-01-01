use {
    super::{error::Result, BinaryParser},
    std::io::Read,
    wrausmt_runtime::syntax::{Resolved, StartField},
};

/// Read the tables section of a binary module from a std::io::Read.
impl<R: Read> BinaryParser<R> {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    pub(in crate::binary) fn read_start_section(&mut self) -> Result<StartField<Resolved>> {
        Ok(StartField {
            idx: self.read_index_use()?,
        })
    }
}
