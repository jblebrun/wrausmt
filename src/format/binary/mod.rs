mod leb128;
mod values;
mod imports;
mod exports;
mod code;
mod countread;
mod types;
mod funcs;
mod custom;
mod ensure_consumed;

use std::io::Read;
use crate::module::{Module, Import, Export, Function, TypeIndex};
use crate::types::FunctionType;
use crate::error::{ResultFrom, Error, Result};
use countread::CountRead;
use code::ReadCode;
use custom::ReadCustom;
use types::ReadTypes;
use imports::ReadImports;
use exports::ReadExports;
use funcs::ReadFuncs;
use leb128::ReadLeb128;
use ensure_consumed::EnsureConsumed;

#[derive(Default)]
pub struct ReadState {
    pub functypes: Option<Box<[TypeIndex]>>
}

/// Read and return the next section in a binary module being read by a std::io::Read
/// If the end of the binary module has been reached, Section::Eof will be returned.
fn read_section<R:Read>(reader: &mut R, state: &ReadState) -> Result<Section> {
    let section_num = match reader.bytes().next() {
        Some(Ok(v)) => v,
        Some(Err(e)) => return Err(e).wrap("parsing section"),
        None => return Ok(Section::Eof)
    };

    let len = reader.read_leb_128().wrap("parsing length")?;
    println!("SECTION {} ({:x}) -- LENGTH {}", section_num, section_num, len);
    let mut section_reader = reader.take(len as u64);
    let section = match section_num {
        0 => Section::Custom(section_reader.read_custom_section().wrap("reading custom")?),
        1 => Section::Types(section_reader.read_types_section().wrap("reading types")?),
        2 => Section::Imports(section_reader.read_imports_section().wrap("reading imports")?),
        3 => Section::Funcs(section_reader.read_funcs_section().wrap("reading funcs")?),
        7 => Section::Exports(section_reader.read_exports_section().wrap("reading exports")?),
        10 => {
            match &state.functypes {
                Some(ft) => Section::Code(section_reader.read_code_section(ft).wrap("reading code")?),
                _ => return Err(Error::new("Received code section without types section".to_string()))
            }
        },
        _ => { section_reader.read_custom_section().wrap("while skipping section")?; Section::Skip }
    };
    
    section_reader.ensure_consumed().wrap(&format!("Section {}", section_num))?;
    
    Ok(section)
}

/// Attempt to interpret the data in the provided std::io:Read as a WASM binary module.
/// If an error occurs, a ParseError will be returned containing the portion of the
/// module that was successfully decoded.
pub fn parse<R>(src: &mut R) -> std::result::Result<Module, ParseError> where R : Read {
    let reader = &mut CountRead::new(src);

    let mut buf: [u8; 4] = [0; 4];
    reader.read_exact(&mut buf).wrap("reading magic")?;
    assert_eq!(buf, [0x00, 0x61, 0x73, 0x6D]);
    reader.read_exact(&mut buf).wrap("reading version")?;
    assert_eq!(buf, [0x01, 0x00, 0x00, 0x00]);

    let mut module = Module::default();
    let mut state = ReadState::default();

    loop {
        let section = read_section(reader, &state);
        match section {
            Ok(Section::Eof) => break,
            Ok(Section::Skip) => (),
            Ok(Section::Custom(_)) => (),
            Ok(Section::Types(t)) => module.types = t,
            Ok(Section::Imports(i)) => module.imports = i,
            Ok(Section::Funcs(f)) => state.functypes = Some(f),
            Ok(Section::Exports(e)) => module.exports = e,
            Ok(Section::Code(c)) => module.funcs = c,
            Err(e) => {
                let consumed = reader.consumed();
                println!("\n\nError parsing at {} (0x{:x}), caused by:\n{}\n\n", consumed, consumed, e);
                return Err(ParseError { cause: e, location:consumed, module: Some(module) })
            }
        }
    }
    Ok(module)
}

#[derive(Debug)]
pub struct ParseError {
    cause: Error,
    location: usize,
    module: Option<Module>
}

impl From<Error> for ParseError {
    fn from(e: Error) -> ParseError {
        ParseError { cause: e, location: 0, module:None }
    }

}
impl std::fmt::Display for ParseError {
    fn fmt<'l>(&'l self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error parsing binary module at {}\nContents so far:{:?}", self.location, self.module)
    }
}

impl std::error::Error for ParseError {}


pub enum Section {
    Eof,
    Skip,
    Custom(Box<[u8]>),
    Types(Box<[FunctionType]>),
    Imports(Box<[Import]>),
    Funcs(Box<[TypeIndex]>),
    Exports(Box<[Export]>),
    Code(Box<[Function]>),
}


