use {
    super::{error::Result, BinaryParser, ParserReader},
    crate::{
        binary::{
            error::{BinaryParseErrorKind, ParseResult},
            leb128::ReadLeb128,
        },
        pctx,
    },
    std::io::Read,
    wrausmt_runtime::syntax,
};

impl<R: ParserReader> BinaryParser<R> {
    /// Read and return the next section in a binary module being read by a
    /// std::io::Read If the end of the binary module has been reached,
    /// Section::Eof will be returned.
    ///
    /// [Spec]: https://webassembly.github.io/spec/core/binary/modules.html#sections
    pub(in crate::binary) fn read_section(&mut self) -> Result<Section> {
        pctx!(self, "read section");
        let section_num = match (&mut self.reader).bytes().next() {
            Some(Ok(v)) => v,
            Some(Err(e)) => Err(e).result(self)?,
            None => return Ok(Section::Eof),
        };

        let expected_size = self.read_u32_leb_128().result(self)? as usize;
        let (section, amount_read) = self.count_reads(|s| {
            Ok(match section_num {
                0 => Section::Custom(s.read_custom_section(expected_size)?),
                1 => Section::Types(s.read_types_section()?),
                2 => Section::Imports(s.read_imports_section()?),
                3 => Section::Funcs(s.read_funcs_section()?),
                4 => Section::Tables(s.read_tables_section()?),
                5 => Section::Mems(s.read_mems_section()?),
                6 => Section::Globals(s.read_globals_section()?),
                7 => Section::Exports(s.read_exports_section()?),
                8 => Section::Start(s.read_start_section()?),
                9 => Section::Elems(s.read_elems_section()?),
                10 => Section::Code(s.read_code_section()?),
                11 => Section::Data(s.read_data_section()?),
                12 => Section::DataCount(s.read_data_count_section()?),
                _ => Err(s.err(BinaryParseErrorKind::MalformedSectionId(section_num)))?,
            })
        })?;

        // It's safe here.
        match amount_read {
            cnt if cnt < expected_size => Err(self.err(BinaryParseErrorKind::SectionTooShort)),
            cnt if cnt > expected_size => Err(self.err(BinaryParseErrorKind::SectionTooLong)),
            _ => Ok(section),
        }
    }
}

#[derive(Debug)]
pub enum Section {
    Eof,
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
