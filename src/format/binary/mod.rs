mod code;
mod countread;
mod custom;
mod data;
mod elems;
mod ensure_consumed;
mod exports;
mod funcs;
mod globals;
mod imports;
mod leb128;
mod mems;
mod section;
mod start;
mod tables;
mod types;
mod values;

use super::error::ParseError;
use crate::{
    err,
    error::Result,
    module::{index, Function, Module, section::Section},
};
use countread::CountRead;
use section::read_section;
use std::io::Read;
use values::ReadWasmValues;

fn resolve_functypes(funcs: &mut [Function], functypes: &[index::Func]) -> Result<()> {
    // In a valid module, we will have parsed the func types section already, so we'll
    // have some partially-initialized function items ready.
    if funcs.len() != functypes.len() {
        return err!("func size mismatch");
    }

    // Add the functype type to the returned function structs.
    for (i, func) in funcs.iter_mut().enumerate() {
        func.functype = functypes[i];
    }
    Ok(())
}

/// Inner parse method accepts a mutable module, so that the outer parse method
/// can return partial module results (useful for debugging).
fn parse_inner<R: Read>(reader: &mut R, module: &mut Module) -> Result<()> {
    reader.read_magic()?;
    reader.read_version()?;

    let mut functypes: Box<[index::Func]> = Box::new([]);

    loop {
        let section = read_section(reader);
        match section {
            Ok(Section::Eof) => break,
            Ok(Section::Skip) => (),
            Ok(Section::Custom(_)) => (),
            Ok(Section::Types(t)) => module.types = t,
            Ok(Section::Imports(i)) => module.imports = i,
            Ok(Section::Funcs(f)) => functypes = f,
            Ok(Section::Tables(t)) => module.tables = t,
            Ok(Section::Mems(m)) => module.mems = m,
            Ok(Section::Globals(g)) => module.globals = g,
            Ok(Section::Exports(e)) => module.exports = e,
            Ok(Section::Start(s)) => module.start = s,
            Ok(Section::Elems(e)) => module.elems = e,
            Ok(Section::Code(c)) => {
                module.funcs = c;
                resolve_functypes(module.funcs.as_mut(), &functypes)?
            }
            Ok(Section::Data(d)) => module.datas = d,
            Ok(Section::DataCount(_)) => {
                // Validate data count
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Attempt to interpret the data in the provided std::io:Read as a WASM binary module.
/// If an error occurs, a ParseError will be returned containing the portion of the
/// module that was successfully decoded.
pub fn parse<R>(src: &mut R) -> std::result::Result<Module, ParseError>
where
    R: Read,
{
    let reader = &mut CountRead::new(src);

    let mut module = Module::default();

    match parse_inner(reader, &mut module) {
        Ok(()) => Ok(module),
        Err(e) => Err(ParseError::new(e, reader.consumed(), module)),
    }
}
