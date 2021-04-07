use super::values::ReadWasmValues;
use crate::error::{Error, Result, ResultFrom};
use crate::module::{Export, ExportDesc};

/// A trait to allow parsing of an exports section from something implementing 
/// std::io::Read.
pub trait ReadExports : ReadWasmValues {
    /// Read the exports section of a module.
    /// exportsec := section vec(export)
    /// export := nm:name d:exportdesc
    /// exportdesc := t:type i:idx_T
    /// 0x00 Func
    /// 0x01 Table
    /// 0x02 Memory
    /// 0x03 Global
    fn read_exports_section(&mut self) -> Result<Box<[Export]>> {
        let items = self.read_leb_128().wrap("parsing count")?;

        (0..items).map(|_| {
            Ok(Export {
                name: self.read_name().wrap("parsing name")?, 
                desc: {
                    let kind = self.read_byte().wrap("parsing kind")?;
                    match kind {
                        0 => ExportDesc::Func(self.read_leb_128().wrap("parsing func")?),
                        1 => ExportDesc::Table(self.read_leb_128().wrap("parsing table")?),
                        2 => ExportDesc::Memory(self.read_leb_128().wrap("parsing memory")?),
                        3 => ExportDesc::Global(self.read_leb_128().wrap("parsing global")?),
                        _ => return Err(Error::new(format!("unknown import desc {}", kind)))
                    }
                }
            })

        }).collect()
    } 
}

impl <I:ReadWasmValues> ReadExports for I {}
