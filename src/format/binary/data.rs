use super::{code::ReadCode, values::ReadWasmValues};
use crate::{
    error::{Result, ResultFrom},
    module::{Data, DataMode},
};

/// Read the tables section of a binary module from a std::io::Read.
pub trait ReadData: ReadWasmValues + ReadCode {
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_data_section(&mut self) -> Result<Box<[Data]>> {
        self.read_vec(|_, s| {
            let variants = s.read_u32_leb_128().wrap("parsing item count")?;
            let active = (variants & 0x01) == 0;
            let active_memidx = (variants & 0x02) != 0;

            let mode = if active {
                let memidx = if active_memidx {
                    s.read_u32_leb_128().wrap("parsing memidx")?
                } else {
                    0
                };
                DataMode::Active {
                    idx: memidx,
                    offset: s.read_expr()?,
                }
            } else {
                DataMode::Passive
            };

            Ok(Data {
                init: Box::new([]),
                mode,
            })
        })
    }

    fn read_data_count_section(&mut self) -> Result<u32> {
        self.read_u32_leb_128().wrap("parsing data count")
    }
}

impl<I: ReadWasmValues + ReadCode> ReadData for I {}
