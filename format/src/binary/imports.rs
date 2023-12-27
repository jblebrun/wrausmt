use {
    super::{
        error::{BinaryParseError, Result, WithContext},
        values::ReadWasmValues,
    },
    wrausmt::syntax::{ImportDesc, ImportField, Resolved},
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
    fn read_imports_section(&mut self) -> Result<Vec<ImportField<Resolved>>> {
        self.read_vec(|_, s| {
            Ok(ImportField {
                id:      None,
                modname: s.read_name().ctx("parsing module name")?,
                name:    s.read_name().ctx("parsing name")?,
                exports: vec![],
                desc:    {
                    let kind = s.read_byte().ctx("parsing kind")?;
                    match kind {
                        0 => ImportDesc::Func(s.read_type_use().ctx("parsing func")?),
                        1 => ImportDesc::Table(s.read_table_type().ctx("parsing table")?),
                        2 => ImportDesc::Mem(s.read_memory_type().ctx("parsing memory")?),
                        3 => ImportDesc::Global(s.read_global_type().ctx("parsing global")?),
                        _ => return Err(BinaryParseError::InvalidImportType(kind)),
                    }
                },
            })
        })
    }
}

impl<I: ReadWasmValues> ReadImports for I {}
