use {
    super::{error::Result, BinaryParser},
    crate::{
        binary::{error::ParseResult, leb128::ReadLeb128, EnsureConsumed},
        pctx,
    },
    std::io::Read,
    wrausmt_runtime::syntax,
};

impl<R: Read> BinaryParser<R> {
    /// Read and return the next section in a binary module being read by a
    /// std::io::Read If the end of the binary module has been reached,
    /// Section::Eof will be returned.
    ///
    /// [Spec]: https://webassembly.github.io/spec/core/binary/modules.html#sections
    pub(in crate::binary) fn read_section(&mut self) -> Result<Section> {
        pctx!(self, "read section");
        let section_num = match (&mut self.reader).bytes().next() {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Err(e).result(self)?,
            None => return Ok(Section::Eof),
        };

        let len = self.read_u32_leb_128().result(self)?;
        let mut section_reader = self.limited(len as u64);
        let section = match section_num {
            0 => Section::Custom(section_reader.read_custom_section()?),
            1 => Section::Types(section_reader.read_types_section()?),
            2 => Section::Imports(section_reader.read_imports_section()?),
            3 => Section::Funcs(section_reader.read_funcs_section()?),
            4 => Section::Tables(section_reader.read_tables_section()?),
            5 => Section::Mems(section_reader.read_mems_section()?),
            6 => Section::Globals(section_reader.read_globals_section()?),
            7 => Section::Exports(section_reader.read_exports_section()?),
            8 => Section::Start(section_reader.read_start_section()?),
            9 => Section::Elems(section_reader.read_elems_section()?),
            10 => Section::Code(section_reader.read_code_section()?),
            11 => Section::Data(section_reader.read_data_section()?),
            12 => Section::DataCount(section_reader.read_data_count_section()?),
            _ => {
                section_reader.read_custom_section()?;
                Section::Skip
            }
        };

        section_reader.ensure_consumed()?;

        Ok(section)
    }
}

#[derive(Debug)]
pub enum Section {
    Eof,
    Skip,
    Custom(syntax::CustomField),
    Types(Vec<syntax::TypeField>),
    Imports(Vec<syntax::ImportField<syntax::Resolved>>),
    Funcs(Vec<syntax::Index<syntax::Resolved, syntax::TypeIndex>>),
    Tables(Vec<syntax::TableField>),
    Mems(Vec<syntax::MemoryField>),
    Globals(Vec<syntax::GlobalField<syntax::Resolved>>),
    Exports(Vec<syntax::ExportField<syntax::Resolved>>),
    Start(syntax::StartField<syntax::Resolved>),
    Elems(Vec<syntax::ElemField<syntax::Resolved>>),
    Code(Vec<syntax::FuncField<syntax::Resolved>>),
    Data(Vec<syntax::DataField<syntax::Resolved>>),
    DataCount(u32),
}
