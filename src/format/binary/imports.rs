use super::values::ReadWasmValues;
use crate::{
    err,
    error::{Result, ResultFrom},
    module::{Import, ImportDesc},
};

/// A trait to allow parsing of an imports section from something implementing
/// std::io::Read.
pub trait ReadImports: ReadWasmValues {
    /// Read the imports section of a module.
    /// importsec := section vec(import)
    /// import := modname:name nm:name d:exportdesc
    /// exportdesc :=
    /// 0x00 (func) i:typeidx
    /// 0x01 (table) tt:tabletype
    /// 0x02 (memory) mt:memorytype
    /// 0x03 (global) gt:globaltype
    fn read_imports_section(&mut self) -> Result<Box<[Import]>> {
        self.read_vec(|_, s| {
            Ok(Import {
                module_name: s.read_name().wrap("parsing module name")?,
                name: s.read_name().wrap("parsing name")?,
                desc: {
                    let kind = s.read_byte().wrap("parsing kind")?;
                    match kind {
                        0 => ImportDesc::Func(s.read_u32_leb_128().wrap("parsing func")?),
                        1 => ImportDesc::Table(s.read_table_type().wrap("parsing table")?),
                        2 => ImportDesc::Memory(s.read_memory_type().wrap("parsing memory")?),
                        3 => ImportDesc::Global(s.read_global_type().wrap("parsing global")?),
                        _ => return err!("unknown import desc {}", kind),
                    }
                },
            })
        })
    }
}

impl<I: ReadWasmValues> ReadImports for I {}
