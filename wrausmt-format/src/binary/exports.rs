use {
    super::{
        error::{BinaryParseErrorKind, Result},
        BinaryParser, ParserReader,
    },
    crate::{binary::read_with_location::Locate, pctx},
    wrausmt_runtime::syntax::{ExportDesc, ExportField, Resolved},
};

/// A trait to allow parsing of an exports section from something implementing
/// std::io::Read.
impl<R: ParserReader> BinaryParser<R> {
    /// Read the exports section of a module.
    /// exportsec := section vec(export)
    /// export := nm:name d:exportdesc
    /// exportdesc := t:type i:idx_T
    /// 0x00 Func
    /// 0x01 Table
    /// 0x02 Memory
    /// 0x03 Global
    pub(in crate::binary) fn read_exports_section(&mut self) -> Result<Vec<ExportField<Resolved>>> {
        self.read_vec(|_, s| s.read_export_field())
    }

    fn read_export_desc(&mut self) -> Result<ExportDesc<Resolved>> {
        pctx!(self, "read exprt desc");
        let kind = self.read_byte()?;
        match kind {
            0 => Ok(ExportDesc::Func(self.read_index_use()?)),
            1 => Ok(ExportDesc::Table(self.read_index_use()?)),
            2 => Ok(ExportDesc::Mem(self.read_index_use()?)),
            3 => Ok(ExportDesc::Global(self.read_index_use()?)),
            _ => Err(self.err(BinaryParseErrorKind::InvalidExportType(kind))),
        }
    }

    fn read_export_field(&mut self) -> Result<ExportField<Resolved>> {
        pctx!(self, "read exprt field");
        let location = self.location();
        Ok(ExportField {
            name: self.read_name()?,
            exportdesc: self.read_export_desc()?,
            location,
        })
    }
}
