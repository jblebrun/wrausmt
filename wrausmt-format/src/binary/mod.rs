use {
    self::{error::BinaryParseError, read_with_location::ReadWithLocation},
    crate::{
        pctx,
        tracer::{TraceDropper, Tracer},
    },
    std::{cell::RefCell, io::Take, rc::Rc},
};

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
mod read_with_location;
mod section;
mod start;
mod tables;
mod types;
mod values;

use {
    crate::binary::{error::BinaryParseErrorKind, section::Section},
    error::Result,
    std::io::Read,
    wrausmt_runtime::syntax::{FuncField, Index, Module, Resolved, TypeIndex},
};

impl<R: Read> BinaryParser<R> {
    /// Inner parse method accepts a mutable module, so that the outer parse
    /// method can return partial module results (useful for debugging).
    fn parse(&mut self) -> Result<Module<Resolved>> {
        let mut module = Module::default();

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
                    self.resolve_functypes(module.funcs.as_mut(), &functypes)?
                }
                Section::Data(d) => module.data = d,
                Section::DataCount(c) => {
                    if module.data.len() != c as usize {
                        return Err(self.err(BinaryParseErrorKind::DataCountMismatch));
                    }
                }
            }
        }
        Ok(module)
    }

    pub(in crate::binary) fn fctx(&self, msg: &str) -> TraceDropper {
        self.tracer.borrow_mut().trace(msg)
    }

    fn resolve_functypes(
        &mut self,
        funcs: &mut [FuncField<Resolved>],
        functypes: &[Index<Resolved, TypeIndex>],
    ) -> Result<()> {
        // In a valid module, we will have parsed the func types section already, so
        // we'll have some partially-initialized function items ready.
        if funcs.len() != functypes.len() {
            return Err(self.err(BinaryParseErrorKind::FuncSizeMismatch));
        }

        // Add the functype type to the returned function structs.
        for (i, func) in funcs.iter_mut().enumerate() {
            func.typeuse.typeidx = Some(functypes[i].clone());
        }
        Ok(())
    }
}

pub struct BinaryParser<R: Read + Sized> {
    reader: R,
    tracer: Rc<RefCell<Tracer>>,
}

impl<R: Read> BinaryParser<R> {
    pub fn new(reader: R) -> Self {
        BinaryParser {
            reader,
            tracer: Rc::new(RefCell::new(Tracer::new("binary parser"))),
        }
    }

    pub fn limited(&mut self, limit: u64) -> BinaryParser<Take<&mut R>> {
        BinaryParser {
            reader: self.reader.by_ref().take(limit),
            tracer: self.tracer.clone(),
        }
    }

    // We use this to create errors that capture the current Tracer context into the
    // error. If we tried to do it any later than error creation time, the error
    // would be cleared.
    pub(in crate::binary) fn err(&self, kind: BinaryParseErrorKind) -> BinaryParseError {
        BinaryParseError::new(kind, self.tracer.borrow().clone_msgs())
    }

    fn take_reader(self) -> R {
        self.reader
    }
}

pub trait EnsureConsumed<R> {
    fn ensure_consumed(&self) -> Result<()>;
}

impl<R: Read> EnsureConsumed<BinaryParser<Take<R>>> for BinaryParser<Take<R>> {
    fn ensure_consumed(&self) -> Result<()> {
        pctx!(self, "ensure consumed");
        let remaining = self.reader.limit();
        if remaining > 0 {
            Err(self.err(BinaryParseErrorKind::ExtraSectionBytes(remaining)))
        } else {
            Ok(())
        }
    }
}

impl<R: Read> Read for BinaryParser<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

/// Attempt to interpret the data in the provided std::io:Read as a WASM binary
/// module. If an error occurs, a ParseError will be returned containing the
/// portion of the module that was successfully decoded.
pub fn parse_wasm_data(src: &mut impl Read) -> Result<Module<Resolved>> {
    // TODO: Is it possible to structure this so that BinaryParser creates a
    // ReaderWithLocation internally from the source, and then populated into
    // the error as part of the err method? Needing to re-wrap the reader in
    // Take made this complicated.
    let reader = ReadWithLocation::new(src);
    let mut parser = BinaryParser::new(reader);
    parser
        .parse()
        .map_err(|e| e.with_location(parser.take_reader().location()))
}
