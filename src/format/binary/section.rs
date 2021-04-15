use super::{
    code::ReadCode, custom::ReadCustom, data::ReadData, elems::ReadElems,
    ensure_consumed::EnsureConsumed, exports::ReadExports, funcs::ReadFuncs, globals::ReadGlobals,
    imports::ReadImports, start::ReadStart, tables::ReadTables, mems::ReadMems,
    types::ReadTypes,
    values::ReadWasmValues
};
use crate::{
    module::Section,
    error::{Result, ResultFrom},
};
use std::io::Read;

pub trait SectionReader : ReadWasmValues + ReadCode  {
    /// Read and return the next section in a binary module being read by a std::io::Read
    /// If the end of the binary module has been reached, Section::Eof will be returned.
    ///
    /// [Spec]: https://webassembly.github.io/spec/core/binary/modules.html#sections
    fn read_section(&mut self) -> Result<Section> {
        let section_num = match self.bytes().next() {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Err(e).wrap("parsing section"),
            None => return Ok(Section::Eof),
        };

        let len = self.read_u32_leb_128().wrap("parsing length")?;
        println!(
            "SECTION {} ({:x}) -- LENGTH {}",
            section_num, section_num, len
        );
        let mut section_reader = self.take(len as u64);
        let section = match section_num {
            0 => Section::Custom(
                section_reader
                .read_custom_section()
                .wrap("reading custom")?,
            ),
            1 => Section::Types(section_reader.read_types_section().wrap("reading types")?),
            2 => Section::Imports(
                section_reader
                .read_imports_section()
                .wrap("reading imports")?,
            ),
            3 => Section::Funcs(section_reader.read_funcs_section().wrap("reading funcs")?),
            4 => Section::Tables(
                section_reader
                .read_tables_section()
                .wrap("reading tables")?,
            ),
            5 => Section::Mems(section_reader.read_mems_section().wrap("reading mems")?),
            6 => Section::Globals(
                section_reader
                .read_globals_section()
                .wrap("reading globals")?,
            ),
            7 => Section::Exports(
                section_reader
                .read_exports_section()
                .wrap("reading exports")?,
            ),
            8 => Section::Start(section_reader.read_start_section().wrap("reading start")?),
            9 => Section::Elems(section_reader.read_elems_section().wrap("reading elems")?),
            10 => Section::Code(section_reader.read_code_section().wrap("reading code")?),
            11 => Section::Data(section_reader.read_data_section().wrap("reading data")?),
            12 => Section::DataCount(
                section_reader
                .read_data_count_section()
                .wrap("reading data count")?,
            ),
            _ => {
                section_reader
                    .read_custom_section()
                    .wrap("while skipping section")?;
                Section::Skip
            }
        };

        section_reader
            .ensure_consumed()
            .wrap(&format!("Section {}", section_num))?;

        Ok(section)
    }
}

impl <R:ReadWasmValues + ReadCode> SectionReader for R {}
