use crate::{
    module::{Import, ImportDesc},
    types::{Limits, MemType, GlobalType, TableType},
    error::{Result, ResultFrom},
    err
};
use super::values::ReadWasmValues;

/// A trait to allow parsing of an imports section from something implementing 
/// std::io::Read.
pub trait ReadImports : ReadWasmValues {
    /// Read the imports section of a module.
    /// importsec := section vec(import)
    /// import := modname:name nm:name d:exportdesc
    /// exportdesc := 
    /// 0x00 (func) i:typeidx
    /// 0x01 (table) tt:tabletype
    /// 0x02 (memory) mt:memorytype
    /// 0x03 (global) gt:globaltype
    fn read_imports_section(&mut self) -> Result<Box<[Import]>> {
        let items = self.read_u32_leb_128().wrap("parsing count")?;

        (0..items).map(|_| {
            Ok(Import {
                module_name: self.read_name().wrap("parsing module name")?,
                name: self.read_name().wrap("parsing name")?, 
                desc: {
                    let kind = self.read_byte().wrap("parsing kind")?;
                    match kind {
                        0 => ImportDesc::Func(self.read_u32_leb_128().wrap("parsing func")?),
                        1 => ImportDesc::Table(self.read_table_type().wrap("parsing table")?),
                        2 => ImportDesc::Memory(self.read_memory_type().wrap("parsing memory")?),
                        3 => ImportDesc::Global(self.read_global_type().wrap("parsing global")?),
                        _ => return err!("unknown import desc {}", kind)
                    }
                }
            })

        }).collect()
    }

    fn read_memory_type(&mut self) -> Result<MemType> {
        Ok(MemType {
            limits: self.read_limits().wrap("parsing limits")?
        })
    }


    fn read_table_type(&mut self) -> Result<TableType> {
        Ok(TableType {
            reftype: self.read_ref_type().wrap("parsing reftype")?,
            limits: self.read_limits().wrap("parsing limits")?
        })
    }

    fn read_global_type(&mut self) -> Result<GlobalType> {
        Ok(GlobalType {
            valtype: self.read_value_type().wrap("parsing value")?,
            mutable: self.read_bool().wrap("parsing mutable")?,
        })
    }

    fn read_limits(&mut self) -> Result<Limits> {
        let has_upper = self.read_bool().wrap("parsing has upper")?;
        Ok(Limits {
            lower: self.read_u32_leb_128().wrap("parsing lower")?,
            upper: if has_upper { Some(self.read_u32_leb_128().wrap("parsing upper")?) } else { None }
        })
    }
}

impl <I:ReadWasmValues> ReadImports for I {}
