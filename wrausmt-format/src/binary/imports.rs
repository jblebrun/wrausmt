use {
    super::{
        error::{BinaryParseErrorKind, Result},
        BinaryParser, ParserReader,
    },
    crate::{binary::read_with_location::Locate, pctx},
    wrausmt_runtime::syntax::{ImportDesc, ImportField, Resolved, Unvalidated},
};

/// A trait to allow parsing of an imports section from something implementing
/// std::io::Read.
impl<R: ParserReader> BinaryParser<R> {
    /// Read the imports section of a module.
    /// importsec := section vec(import)
    /// import := modname:name nm:name d:exportdesc
    /// exportdesc :=
    /// 0x00 (func) i:typeidx
    /// 0x01 (table) tt:tabletype
    /// 0x02 (memory) mt:memorytype
    /// 0x03 (global) gt:globaltype
    pub(in crate::binary) fn read_imports_section(
        &mut self,
    ) -> Result<Vec<ImportField<Resolved, Unvalidated>>> {
        pctx!(self, "read imports section");
        let location = self.location();
        self.read_vec(|_, s| {
            Ok(ImportField {
                id: None,
                modname: s.read_name()?,
                name: s.read_name()?,
                exports: vec![],
                desc: {
                    let kind = s.read_byte()?;
                    match kind {
                        0 => ImportDesc::Func(s.read_type_use()?),
                        1 => ImportDesc::Table(s.read_table_type()?),
                        2 => ImportDesc::Mem(s.read_memory_type()?),
                        3 => ImportDesc::Global(s.read_global_type()?),
                        _ => return Err(s.err(BinaryParseErrorKind::MalformedImportKind(kind))),
                    }
                },
                location,
            })
        })
    }
}
