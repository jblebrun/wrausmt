use {
    super::{
        error::{Result, WithContext},
        BinaryParser,
    },
    std::io::Read,
    wrausmt_runtime::syntax::{Index, Resolved, TypeIndex},
};

/// Read the funcs section of a binary module from a std::io::Read.
impl<R: Read> BinaryParser<R> {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    pub(in crate::binary) fn read_funcs_section(
        &mut self,
    ) -> Result<Vec<Index<Resolved, TypeIndex>>> {
        self.read_vec(|_, s| s.read_index_use().ctx("parsing func"))
    }
}
