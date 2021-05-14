use super::{code::ReadCode, values::ReadWasmValues};
use crate::{
    error::{Result, ResultFrom},
    syntax::{DataField, DataInit, Index, Resolved},
};

/// Read the tables section of a binary module from a std::io::Read.
pub trait ReadData: ReadWasmValues + ReadCode {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_data_section(&mut self) -> Result<Vec<DataField<Resolved>>> {
        self.read_vec(|_, s| s.read_data_field())
    }

    fn read_data_field(&mut self) -> Result<DataField<Resolved>> {
        let variants = self.read_u32_leb_128().wrap("parsing item count")?;
        let active = (variants & 0x01) == 0;
        let active_memidx = (variants & 0x02) != 0;

        let init = if active {
            let memidx = if active_memidx {
                self.read_index_use().wrap("parsing memidx")?
            } else {
                Index::unnamed(0)
            };
            Some(DataInit {
                memidx,
                offset: self.read_expr()?,
            })
        } else {
            None
        };

        let data = self.read_bytes()?.to_vec();

        Ok(DataField {
            id: None,
            data,
            init,
        })
    }

    fn read_data_count_section(&mut self) -> Result<u32> {
        self.read_u32_leb_128().wrap("parsing data count")
    }
}

impl<I: ReadWasmValues + ReadCode> ReadData for I {}
