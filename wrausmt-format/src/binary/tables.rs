use {
    super::{error::Result, BinaryParser, ParserReader},
    crate::pctx,
    wrausmt_runtime::syntax::{TableField, Unvalidated},
};

/// Read the tables section of a binary module from a std::io::Read.
impl<R: ParserReader> BinaryParser<R> {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    pub(in crate::binary) fn read_tables_section(
        &mut self,
    ) -> Result<Vec<TableField<Unvalidated>>> {
        pctx!(self, "read tables section");
        self.read_vec(|_, s| s.read_table_field())
    }

    fn read_table_field(&mut self) -> Result<TableField<Unvalidated>> {
        pctx!(self, "read table field");
        let location = self.reader.location();
        Ok(TableField {
            id: None,
            tabletype: self.read_table_type()?,
            exports: vec![],
            location,
        })
    }
}
