use {
    super::{
        error::{Result, WithContext},
        BinaryParser,
    },
    std::io::Read,
    wrausmt_runtime::syntax::TableField,
};

/// Read the tables section of a binary module from a std::io::Read.
impl<R: Read> BinaryParser<R> {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    pub(in crate::binary) fn read_tables_section(&mut self) -> Result<Vec<TableField>> {
        self.read_vec(|_, s| s.read_table_field().ctx("read table type"))
    }

    fn read_table_field(&mut self) -> Result<TableField> {
        Ok(TableField {
            id:        None,
            tabletype: self.read_table_type()?,
            exports:   vec![],
        })
    }
}
