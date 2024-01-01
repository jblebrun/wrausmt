use {self::tokenizer::Tokenizer, std::io::Take};

pub mod error;

mod code;
mod custom;
mod data;
mod elems;
mod exports;
mod funcs;
mod globals;
mod imports;
/// This module contains the logic for parsing a WebAssembly module represented
/// using the binary format in the specification. The parsing strategy is
/// straightforward, and results in a [Module] being returned. There's currently
/// no AST or other form of intermediate representation; the parser directly
/// generates the internal type used by the execution engine. This may changein
/// the future.
///
/// The code is organized into modules that implement various sub-aspects of the
/// binary parsing task as traits on [std::io::Read].
pub mod leb128;
mod mems;
mod section;
mod start;
mod tables;
mod tokenizer;
mod types;
mod values;

use {
    crate::binary::{error::BinaryParseErrorKind, section::Section},
    error::{Result, WithContext},
    std::io::Read,
    wrausmt_runtime::syntax::{FuncField, Index, Module, Resolved, TypeIndex},
};

fn resolve_functypes(
    funcs: &mut [FuncField<Resolved>],
    functypes: &[Index<Resolved, TypeIndex>],
) -> Result<()> {
    // In a valid module, we will have parsed the func types section already, so
    // we'll have some partially-initialized function items ready.
    if funcs.len() != functypes.len() {
        return Err(BinaryParseErrorKind::FuncSizeMismatch.into());
    }

    // Add the functype type to the returned function structs.
    for (i, func) in funcs.iter_mut().enumerate() {
        func.typeuse.typeidx = Some(functypes[i].clone());
    }
    Ok(())
}

impl<R: Read> BinaryParser<R> {
    /// Inner parse method accepts a mutable module, so that the outer parse
    /// method can return partial module results (useful for debugging).
    fn parse(&mut self, module: &mut Module<Resolved>) -> Result<()> {
        self.read_magic()?;
        self.read_version()?;

        let mut functypes: Vec<Index<Resolved, TypeIndex>> = vec![];

        loop {
            let section = self.read_section()?;
            match section {
                Section::Eof => break,
                Section::Skip => (),
                Section::Custom(_) => (),
                Section::Types(t) => module.types = t,
                Section::Imports(i) => module.imports = i,
                Section::Funcs(f) => functypes = f,
                Section::Tables(t) => module.tables = t,
                Section::Mems(m) => module.memories = m,
                Section::Globals(g) => module.globals = g,
                Section::Exports(e) => module.exports = e,
                Section::Start(s) => module.start = Some(s),
                Section::Elems(e) => module.elems = e,
                Section::Code(c) => {
                    module.funcs = c;
                    resolve_functypes(module.funcs.as_mut(), &functypes)?
                }
                Section::Data(d) => module.data = d,
                Section::DataCount(c) => {
                    if module.data.len() != c as usize {
                        return Err(BinaryParseErrorKind::DataCountMismatch.into());
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct BinaryParser<R: Read + Sized> {
    tokenizer: R,
}

pub trait EnsureConsumed<R> {
    fn ensure_consumed(&self) -> Result<()>;
}

impl<R: Read> EnsureConsumed<BinaryParser<Take<R>>> for BinaryParser<Take<R>> {
    fn ensure_consumed(&self) -> Result<()> {
        let remaining = self.tokenizer.limit();
        if remaining > 0 {
            Err(BinaryParseErrorKind::ExtraSectionBytes(remaining).into())
        } else {
            Ok(())
        }
    }
}

impl<R: Read> Read for BinaryParser<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.tokenizer.read(buf)
    }
}

/// Attempt to interpret the data in the provided std::io:Read as a WASM binary
/// module. If an error occurs, a ParseError will be returned containing the
/// portion of the module that was successfully decoded.
pub fn parse_wasm_data(src: &mut impl Read) -> Result<Module<Resolved>> {
    let tokenizer = Tokenizer::new(src);
    let mut parser = BinaryParser { tokenizer };

    let mut module = Module::default();

    match parser.parse(&mut module) {
        Ok(()) => Ok(module),
        Err(e) => Err(e.ctx(format!("at {:?}", parser.tokenizer.location()))),
    }
}
