use {
    super::{error::Result, leb128::ReadLeb128, BinaryParser, ParserReader},
    crate::{binary::error::ParseResult, pctx},
    wrausmt_runtime::syntax::{DataField, DataInit, Index, Resolved, UncompiledExpr},
};

/// Read the tables section of a binary module from a std::io::Read.
impl<R: ParserReader> BinaryParser<R> {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    pub(in crate::binary) fn read_data_section(
        &mut self,
    ) -> Result<Vec<DataField<Resolved, UncompiledExpr<Resolved>>>> {
        pctx!(self, "read data section");
        self.read_vec(|_, s| s.read_data_field())
    }

    pub(in crate::binary) fn read_data_count_section(&mut self) -> Result<u32> {
        pctx!(self, "read data count section");
        self.read_u32_leb_128().result(self)
    }

    fn read_data_field(&mut self) -> Result<DataField<Resolved, UncompiledExpr<Resolved>>> {
        pctx!(self, "read data field");
        let location = self.reader.location();
        let variants = self.read_u32_leb_128().result(self)?;
        let active = (variants & 0x01) == 0;
        let active_memidx = (variants & 0x02) != 0;

        let init = if active {
            let memidx = if active_memidx {
                self.read_index_use()?
            } else {
                Index::unnamed(0)
            };
            Some(DataInit {
                memidx,
                offset: self.read_expr(false)?,
            })
        } else {
            None
        };

        let data = self.read_bytes()?;

        Ok(DataField {
            id: None,
            data,
            init,
            location,
        })
    }
}
