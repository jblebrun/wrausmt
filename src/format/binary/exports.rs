use {
    super::{
        error::{BinaryParseError, Result, WithContext},
        values::ReadWasmValues,
    },
    crate::syntax::{ExportDesc, ExportField, Resolved},
};

/// A trait to allow parsing of an exports section from something implementing
/// std::io::Read.
pub trait ReadExports: ReadWasmValues {
    /// Read the exports section of a module.
    /// exportsec := section vec(export)
    /// export := nm:name d:exportdesc
    /// exportdesc := t:type i:idx_T
    /// 0x00 Func
    /// 0x01 Table
    /// 0x02 Memory
    /// 0x03 Global
    fn read_exports_section(&mut self) -> Result<Vec<ExportField<Resolved>>> {
        self.read_vec(|_, s| s.read_export_field())
    }

    fn read_export_desc(&mut self) -> Result<ExportDesc<Resolved>> {
        let kind = self.read_byte().ctx("parsing kind")?;
        match kind {
            0 => Ok(ExportDesc::Func(self.read_index_use().ctx("parsing func")?)),
            1 => Ok(ExportDesc::Table(
                self.read_index_use().ctx("parsing table")?,
            )),
            2 => Ok(ExportDesc::Mem(
                self.read_index_use().ctx("parsing memory")?,
            )),
            3 => Ok(ExportDesc::Global(
                self.read_index_use().ctx("parsing global")?,
            )),
            _ => Err(BinaryParseError::InvalidExportType(kind)),
        }
    }

    fn read_export_field(&mut self) -> Result<ExportField<Resolved>> {
        Ok(ExportField {
            name:       self.read_name().ctx("parsing name")?,
            exportdesc: self.read_export_desc()?,
        })
    }
}

impl<I: ReadWasmValues> ReadExports for I {}
