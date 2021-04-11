use std::io::Read;
use crate::{
    module::{Import, Export, Function, index},
    types::FunctionType,
    error::{ResultFrom, Result}
};
use super::{
    code::ReadCode,
    custom::ReadCustom,
    types::ReadTypes,
    imports::ReadImports,
    exports::ReadExports,
    funcs::ReadFuncs,
    leb128::ReadLeb128,
    ensure_consumed::EnsureConsumed,
};

pub enum Section {
    Eof,
    Skip,
    Custom(Box<[u8]>),
    Types(Box<[FunctionType]>),
    Imports(Box<[Import]>),
    Funcs(Box<[index::Type]>),
    Exports(Box<[Export]>),
    Code(Box<[Function]>),
}

/// Read and return the next section in a binary module being read by a std::io::Read
/// If the end of the binary module has been reached, Section::Eof will be returned.
pub fn read_section<R:Read>(reader: &mut R) -> Result<Section> {
    let section_num = match reader.bytes().next() {
        Some(Ok(v)) => v,
        Some(Err(e)) => return Err(e).wrap("parsing section"),
        None => return Ok(Section::Eof)
    };

    let len = reader.read_u32_leb_128().wrap("parsing length")?;
    println!("SECTION {} ({:x}) -- LENGTH {}", section_num, section_num, len);
    let mut section_reader = reader.take(len as u64);
    let section = match section_num {
        0 => Section::Custom(section_reader.read_custom_section().wrap("reading custom")?),
        1 => Section::Types(section_reader.read_types_section().wrap("reading types")?),
        2 => Section::Imports(section_reader.read_imports_section().wrap("reading imports")?),
        3 => Section::Funcs(section_reader.read_funcs_section().wrap("reading funcs")?),
        7 => Section::Exports(section_reader.read_exports_section().wrap("reading exports")?),
        10 => Section::Code(section_reader.read_code_section().wrap("reading code")?),
        _ => { section_reader.read_custom_section().wrap("while skipping section")?; Section::Skip }
    };
    
    section_reader.ensure_consumed().wrap(&format!("Section {}", section_num))?;
    
    Ok(section)
}
