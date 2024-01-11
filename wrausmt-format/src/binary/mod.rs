use {
    self::{
        error::{BinaryParseError, EofAsKind},
        read_with_location::{Location, ReadWithLocation},
    },
    crate::pctx,
    std::{cell::RefCell, rc::Rc},
    wrausmt_common::{
        tracer::{TraceDropper, Tracer},
        true_or::TrueOr,
    },
    wrausmt_runtime::syntax::{TypeUse, UncompiledExpr},
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
    crate::binary::error::BinaryParseErrorKind,
    error::Result,
    std::io::Read,
    wrausmt_runtime::syntax::{FuncField, Index, Module, Resolved, TypeIndex},
};

pub trait ParserReader: Read + Location {}

impl<R: ParserReader> BinaryParser<R> {
    /// Inner parse method accepts a mutable module, so that the outer parse
    /// method can return partial module results (useful for debugging).
    fn parse(&mut self) -> Result<Module<Resolved, UncompiledExpr<Resolved>>> {
        let mut module = Module::default();

        self.read_magic()
            .eof_as_kind(BinaryParseErrorKind::UnexpectedEnd)?;
        self.read_version()
            .eof_as_kind(BinaryParseErrorKind::UnexpectedEnd)?;

        let mut functypes: Vec<Index<Resolved, TypeIndex>> = vec![];
        let mut datacount: Option<u32> = None;

        let mut id = self.read_next_section_id(&mut module.customs)?;
        if id == Some(1) {
            pctx!(self, "types section");
            module.types = self.read_section(Self::read_types_section)?;
            id = self.read_next_section_id(&mut module.customs)?;
        }
        if id == Some(2) {
            pctx!(self, "imports section");
            module.imports = self.read_section(Self::read_imports_section)?;
            id = self.read_next_section_id(&mut module.customs)?;
        }
        if id == Some(3) {
            pctx!(self, "funcs section");
            functypes = self.read_section(Self::read_funcs_section)?;
            id = self.read_next_section_id(&mut module.customs)?;
        }
        if id == Some(4) {
            pctx!(self, "tables section");
            module.tables = self.read_section(Self::read_tables_section)?;
            id = self.read_next_section_id(&mut module.customs)?;
        }
        if id == Some(5) {
            pctx!(self, "mems section");
            module.memories = self.read_section(Self::read_mems_section)?;
            id = self.read_next_section_id(&mut module.customs)?;
        }
        if id == Some(6) {
            pctx!(self, "globals section");
            module.globals = self.read_section(Self::read_globals_section)?;
            id = self.read_next_section_id(&mut module.customs)?;
        }
        if id == Some(7) {
            pctx!(self, "globals section");
            module.exports = self.read_section(Self::read_exports_section)?;
            id = self.read_next_section_id(&mut module.customs)?;
        }
        if id == Some(8) {
            pctx!(self, "start section");
            module.start = Some(self.read_section(Self::read_start_section)?);
            id = self.read_next_section_id(&mut module.customs)?;
        }
        if id == Some(9) {
            pctx!(self, "elems section");
            module.elems = self.read_section(Self::read_elems_section)?;
            id = self.read_next_section_id(&mut module.customs)?;
        }
        if id == Some(12) {
            pctx!(self, "data count section");
            datacount = Some(self.read_section(Self::read_data_count_section)?);
            id = self.read_next_section_id(&mut module.customs)?;
        }
        if id == Some(10) {
            pctx!(self, "code section");
            let data_indices_allowed = datacount.is_some();
            module.funcs = self.read_section(|s| s.read_code_section(data_indices_allowed))?;
            id = self.read_next_section_id(&mut module.customs)?;
        }
        if id == Some(11) {
            pctx!(self, "data section");
            module.data = self.read_section(Self::read_data_section)?;
            id = self.read_next_section_id(&mut module.customs)?;
        }

        (id.is_none())
            .true_or_else(|| self.err(BinaryParseErrorKind::UnexpectedContentAfterEnd))?;

        if let Some(datacount) = datacount {
            (datacount as usize == module.data.len())
                .true_or_else(|| self.err(BinaryParseErrorKind::DataCountMismatch))?;
        }

        self.resolve_functypes(&mut module.funcs, &functypes)?;

        Ok(module)
    }

    pub(in crate::binary) fn fctx(&self, msg: &str) -> TraceDropper {
        self.tracer.borrow_mut().trace(msg)
    }

    fn resolve_functypes(
        &mut self,
        funcs: &mut [FuncField<Resolved, UncompiledExpr<Resolved>>],
        functypes: &[Index<Resolved, TypeIndex>],
    ) -> Result<()> {
        // In a valid module, we will have parsed the func types section already, so
        // we'll have some partially-initialized function items ready.
        (funcs.len() == functypes.len())
            .true_or_else(|| self.err(BinaryParseErrorKind::FuncSizeMismatch))?;

        // Add the functype type to the returned function structs.
        for (i, func) in funcs.iter_mut().enumerate() {
            func.typeuse = TypeUse::ByIndex(functypes[i].clone());
        }
        Ok(())
    }
}

pub struct BinaryParser<R: Read + Location + Sized> {
    reader: R,
    tracer: Rc<RefCell<Tracer>>,
}

impl<R: ParserReader> BinaryParser<R> {
    pub fn new(reader: R) -> Self {
        BinaryParser {
            reader,
            tracer: Rc::new(RefCell::new(Tracer::new("binary parser"))),
        }
    }

    // We use this to create errors that capture the current Tracer context into the
    // error. If we tried to do it any later than error creation time, the error
    // would be cleared.
    pub(in crate::binary) fn err(&self, kind: BinaryParseErrorKind) -> BinaryParseError {
        BinaryParseError::new(kind, self.tracer.borrow().clone_msgs(), self.location())
    }

    // A helper that tracks the amount of data read for the duration of the provided
    // action closure.
    pub fn count_reads<T>(
        &mut self,
        action: impl Fn(&mut Self) -> Result<T>,
    ) -> Result<(T, usize)> {
        let start = self.location();
        Ok((action(self)?, self.location() - start))
    }
}

impl<T: Read> ParserReader for ReadWithLocation<T> {}

impl<T: ParserReader> Read for BinaryParser<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}
impl<T: ParserReader> Location for BinaryParser<T> {
    fn location(&self) -> usize {
        self.reader.location()
    }
}
impl<T: ParserReader> ParserReader for BinaryParser<T> {}

/// Attempt to interpret the data in the provided std::io:Read as a WASM binary
/// module. If an error occurs, a ParseError will be returned containing the
/// portion of the module that was successfully decoded.
pub fn parse_wasm_data(src: &mut impl Read) -> Result<Module<Resolved, UncompiledExpr<Resolved>>> {
    let reader = ReadWithLocation::new(src);
    let mut parser = BinaryParser::new(reader);
    parser.parse()
}
