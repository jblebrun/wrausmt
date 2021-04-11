use super::values::ReadWasmValues;
use crate::error::{Result, ResultFrom};
use crate::err;
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
        self.read_vec(|_, s| {
            Ok(Export {
                name: s.read_name().wrap("parsing name")?, 
                desc: {
                    let kind = s.read_byte().wrap("parsing kind")?;
                    match kind {
                        0 => ExportDesc::Func(s.read_u32_leb_128().wrap("parsing func")?),
                        1 => ExportDesc::Table(s.read_u32_leb_128().wrap("parsing table")?),
                        2 => ExportDesc::Memory(s.read_u32_leb_128().wrap("parsing memory")?),
                        3 => ExportDesc::Global(s.read_u32_leb_128().wrap("parsing global")?),
                        _ => return err!("unknown import desc {:x}", kind)
                    }
                }
            })

        })
    } 
}

impl <I:ReadWasmValues> ReadExports for I {}
