use {
    super::{error::Result, BinaryParser, ParserReader},
    crate::pctx,
    wrausmt_runtime::syntax::{Index, Resolved, TypeIndex},
};

/// Read the funcs section of a binary module from a std::io::Read.
impl<R: ParserReader> BinaryParser<R> {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    pub fn read_funcs_section(&mut self) -> Result<Vec<Index<Resolved, TypeIndex>>> {
        pctx!(self, "read funcs section");
        self.read_vec(|_, s| s.read_index_use())
    }
}
