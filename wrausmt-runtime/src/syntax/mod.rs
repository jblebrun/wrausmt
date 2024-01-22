//! The syntax elements related to parsing a module.
//!
//! [Spec]: https://webassembly.github.io/spec/core/text/modules.html#modules

mod indices;
pub mod location;
pub mod types;

#[cfg(test)]
mod tests;

pub use indices::{
    DataIndex, ElemIndex, FuncIndex, GlobalIndex, IndexSpace, LabelIndex, LocalIndex, MemoryIndex,
    Resolved, ResolvedState, TableIndex, TypeIndex, Unresolved,
};
use {
    self::location::Location,
    std::{
        borrow::Cow,
        fmt::{self, Debug},
        marker::PhantomData,
        slice::SliceIndex,
    },
    types::{GlobalType, MemType, RefType, TableType, ValueType},
};

/// A wasm identifier. Contains only valid `idchar` characters.
#[derive(Clone, Default, Debug, Eq, Hash, PartialEq)]
pub struct Id {
    data: Cow<'static, str>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum IdError {
    InvalidIdChar(u8),
}

impl std::error::Error for IdError {}
impl std::fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

const fn is_idchar(c: u8) -> bool {
    matches!(c,
        b'0'..=b'9' | b'A'..=b'Z'  | b'a'..=b'z' | b'!' | b'#' |
        b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-' | b'/' |
        b':' | b'<' | b'=' | b'>'  | b'?' | b'@' | b'\\' |
        b'^' | b'_' | b'`' | b'|'  | b'~' | b'.'
    )
}

fn validate_chars(bytes: &[u8]) -> Result<(), IdError> {
    match bytes.iter().find(|b| !is_idchar(**b)) {
        Some(b) => Err(IdError::InvalidIdChar(*b)),
        _ => Ok(()),
    }
}
impl Id {
    pub fn literal(s: &'static str) -> Id {
        validate_chars(s.as_bytes()).unwrap();
        Id {
            data: Cow::Borrowed(s),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_bytes()
    }

    pub fn as_str(&self) -> &str {
        &self.data
    }
}

impl TryFrom<&str> for Id {
    type Error = IdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_chars(value.as_bytes())?;
        Ok(Self {
            data: value.to_owned().into(),
        })
    }
}

impl TryFrom<&[u8]> for Id {
    type Error = IdError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // If the utf8 is invalid, the from &str will fail when checking
        // for all idchars.
        unsafe { std::str::from_utf8_unchecked(value) }.try_into()
    }
}

impl TryFrom<Vec<u8>> for Id {
    type Error = IdError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        value.as_slice().try_into()
    }
}

impl PartialEq<str> for Id {
    fn eq(&self, other: &str) -> bool {
        &*self.data == other
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.data)
    }
}

/// Represents one index usage point. It may be named ($id) or numeric. [Spec]
///
/// An `Index<Resolved>` will have the correct numeric value associated.
/// `Index<Unresolved>` may contain a numeric value if one was parsed, but may
/// also contain only a string name and a default zero value.
///
/// [Spec]: https://webassembly.github.io/spec/core/text/modules.html#indices
#[derive(Clone, Default, PartialEq)]
pub struct Index<R: ResolvedState, S: IndexSpace> {
    name:           Id,
    value:          u32,
    resolvedmarker: PhantomData<R>,
    indexmarker:    PhantomData<S>,
}

impl<Idx: SliceIndex<[u8], Output = u8>> std::ops::Index<Idx> for Id {
    type Output = u8;

    fn index(&self, index: Idx) -> &u8 {
        &self.data.as_bytes()[index]
    }
}

impl<S: IndexSpace> From<Index<Resolved, S>> for u32 {
    fn from(idx: Index<Resolved, S>) -> u32 {
        idx.value()
    }
}

impl<R: ResolvedState, S: IndexSpace> Index<R, S> {
    pub fn name(&self) -> &Id {
        &self.name
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn named(name: Id, value: u32) -> Self {
        Index {
            name,
            value,
            resolvedmarker: PhantomData,
            indexmarker: PhantomData,
        }
    }

    pub fn unnamed(value: u32) -> Self {
        Index::named(Id::default(), value)
    }

    pub fn resolved(self, value: u32) -> Index<Resolved, S> {
        Index {
            name: self.name,
            value,
            resolvedmarker: PhantomData {},
            indexmarker: PhantomData {},
        }
    }

    pub fn convert<S2: IndexSpace>(self) -> Index<R, S2> {
        Index {
            name:           self.name,
            value:          self.value,
            resolvedmarker: PhantomData {},
            indexmarker:    PhantomData {},
        }
    }
}

impl<R: ResolvedState, S: IndexSpace> fmt::Debug for Index<R, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.name, self.value)
    }
}

pub trait ExprT {}
#[derive(Default, PartialEq)]
/// A parsed text format module. [Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/text/modules.html#modules
pub struct Module<R: ResolvedState, E> {
    pub id:       Option<Id>,
    pub customs:  Vec<CustomField>,
    pub types:    Vec<TypeField>,
    pub funcs:    Vec<FuncField<R, E>>,
    pub tables:   Vec<TableField>,
    pub memories: Vec<MemoryField>,
    pub imports:  Vec<ImportField<R>>,
    pub exports:  Vec<ExportField<R>>,
    pub globals:  Vec<GlobalField<E>>,
    pub start:    Option<StartField<R>>,
    pub elems:    Vec<ElemField<R, E>>,
    pub data:     Vec<DataField<R, E>>,
}

impl<I: ResolvedState, E: Debug> fmt::Debug for Module<I, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(module")?;
        macro_rules! print_all {
            ( $v:expr ) => {
                for i in $v {
                    write!(f, "\n{:?}", i)?;
                }
            };
        }
        if let Some(id) = &self.id {
            write!(f, "{:?}", id)?;
        }

        print_all!(&self.types);
        print_all!(&self.funcs);
        print_all!(&self.tables);
        print_all!(&self.memories);
        print_all!(&self.globals);
        print_all!(&self.imports);
        print_all!(&self.exports);
        print_all!(&self.globals);
        if let Some(st) = &self.start {
            write!(f, "\n{:?}", st)?;
        }
        print_all!(&self.elems);
        print_all!(&self.data);
        write!(f, "\n)")?;
        write!(f, ".IdentifierContext:")
    }
}

#[derive(PartialEq, Clone, Default)]
pub struct FunctionType {
    pub params:  Vec<FParam>,
    pub results: Vec<FResult>,
}

impl FunctionType {
    pub fn anonymously_equals(&self, other: &Self) -> bool {
        self.params.len() == other.params.len()
            && self.results == other.results
            && self
                .params
                .iter()
                .zip(other.params.iter())
                .all(|(a, b)| a.valuetype == b.valuetype)
    }

    pub fn is_void(&self) -> bool {
        self.params.is_empty() && self.results.is_empty()
    }

    pub fn matches_existing(&self, existing: &Self) -> bool {
        self.is_void() || self.anonymously_equals(existing)
    }
}

impl std::fmt::Debug for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for param in &self.params {
            write!(f, " {:?}", param)?;
        }

        for result in &self.results {
            write!(f, " {:?}", result)?;
        }
        Ok(())
    }
}

/// A [Resolved] TypeUse has not just its index name resolved, but also provides
/// a guarantee that the index value stored corresponds to a type use in this
/// module.
#[derive(PartialEq, Clone)]
pub enum TypeUse<R: ResolvedState> {
    /// Represents a type use that was only `(type $n)`
    ByIndex(Index<R, TypeIndex>),
    // Represents a type use that was only `(param _)* (result _)*`
    AnonymousInline(FunctionType),
    // Represents a type use with everything: `type ($n) (param _)* (result _)*`
    NamedInline {
        functiontype: FunctionType,
        index:        Index<R, TypeIndex>,
    },
}

impl<R: ResolvedState> Default for TypeUse<R> {
    fn default() -> Self {
        TypeUse::ByIndex(Index::unnamed(0))
    }
}

impl TypeUse<Resolved> {
    pub fn index(&self) -> &Index<Resolved, TypeIndex> {
        match self {
            TypeUse::ByIndex(i) => i,
            TypeUse::NamedInline { index, .. } => index,
            TypeUse::AnonymousInline(_) => {
                panic!("improperly resolved typeuse (compiler error) {:?}", self)
            }
        }
    }
}

impl TypeUse<Unresolved> {
    pub fn index(&self) -> Option<&Index<Unresolved, TypeIndex>> {
        match self {
            TypeUse::ByIndex(i) => Some(i),
            TypeUse::NamedInline { index, .. } => Some(index),
            TypeUse::AnonymousInline(_) => None,
        }
    }
}

impl<R: ResolvedState> TypeUse<R> {
    pub fn single_result(valuetype: ValueType) -> Self {
        Self::AnonymousInline(FunctionType {
            results: vec![FResult { valuetype }],
            params:  vec![],
        })
    }

    pub fn function_type(&self) -> Option<&FunctionType> {
        match self {
            TypeUse::ByIndex(_) => None,
            TypeUse::NamedInline { functiontype, .. } => Some(functiontype),
            TypeUse::AnonymousInline(ft) => Some(ft),
        }
    }
}

impl<R: ResolvedState> std::fmt::Debug for TypeUse<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeUse::ByIndex(idx) => write!(f, "(type {idx:?}"),
            TypeUse::NamedInline {
                functiontype,
                index,
            } => write!(f, "(type {index:?}) {functiontype:?}"),
            TypeUse::AnonymousInline(functiontype) => write!(f, "{functiontype:?}"),
        }
    }
}

// param := (param id? valtype)
#[derive(PartialEq, Clone)]
pub struct FParam {
    pub id:        Option<Id>,
    pub valuetype: ValueType,
}

impl std::fmt::Debug for FParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.id {
            Some(id) => write!(f, "(param {} {:?})", id, self.valuetype),
            None => write!(f, "(param {:?})", self.valuetype),
        }
    }
}

// result := (result valtype)
#[derive(PartialEq, Clone, Copy)]
pub struct FResult {
    pub valuetype: ValueType,
}

impl std::fmt::Debug for FResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(result {:?})", self.valuetype)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
// customsec := section_0(custom)
// custom := name byte*
pub struct CustomField {
    pub name:    String,
    pub content: Box<[u8]>,
}

// type := (type id? <functype>)
// functype := (func <param>* <result>*)
#[derive(Clone, PartialEq, Default)]
pub struct TypeField {
    pub id:           Option<Id>,
    pub functiontype: FunctionType,
}

impl std::fmt::Debug for TypeField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(type")?;
        if let Some(id) = &self.id {
            write!(f, " {}", id)?;
        }

        write!(f, " (func")?;

        write!(f, " {:?}", self.functiontype)?;

        write!(f, "))")
    }
}

// func := (func id? <typeuse> <local>* <instr>*)
// instr := sequence of instr, or folded expressions
//
// Abbreviations:
// func := (func id? (export  <name>)*  ...)
// func := (func id? (import <modname> <name>) <typeuse>)
#[derive(PartialEq)]
pub struct FuncField<R: ResolvedState, E> {
    pub id:       Option<Id>,
    pub exports:  Vec<String>,
    pub typeuse:  TypeUse<R>,
    pub locals:   Vec<Local>,
    pub body:     E,
    pub location: Location,
}

impl Default for FuncField<Unresolved, UncompiledExpr<Unresolved>> {
    fn default() -> Self {
        Self {
            id:       Default::default(),
            exports:  Default::default(),
            typeuse:  Default::default(),
            locals:   Default::default(),
            body:     Default::default(),
            location: Location { line: 0, pos: 0 },
        }
    }
}

impl<R: ResolvedState, E: Debug> std::fmt::Debug for FuncField<R, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(func")?;

        if let Some(id) = &self.id {
            write!(f, " {}", id)?;
        }

        for export in &self.exports {
            write!(f, " (export {})", export)?;
        }

        write!(f, " {:?}", self.typeuse)?;

        for local in &self.locals {
            write!(f, " {:?}", local)?;
        }
        write!(f, "\n{:?}", self.body)?;
        write!(f, "\n)")
    }
}

// local := (local id? <valtype>)
#[derive(PartialEq)]
pub struct Local {
    pub id:      Option<Id>,
    pub valtype: ValueType,
}

impl std::fmt::Debug for Local {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.id {
            Some(id) => write!(f, "(local {} {:?})", id, self.valtype),
            None => write!(f, "(local {:?})", self.valtype),
        }
    }
}

#[derive(Debug, PartialEq)]
// table :: = (table id? <tabletype>)
// Abbreviations:
// inline imports/exports
// inline elem
pub struct TableField {
    pub id:        Option<Id>,
    pub exports:   Vec<String>,
    pub tabletype: TableType,
    pub location:  Location,
}

// memory := (memory id? <memtype>)
//
// Abbreviations:
// Inline import/export
// Inline data segments
#[derive(Debug, PartialEq)]
pub struct MemoryField {
    pub id:       Option<Id>,
    pub exports:  Vec<String>,
    pub memtype:  MemType,
    pub location: Location,
}

// global := (global <id>? <globaltype> <expr>)
#[derive(PartialEq)]
pub struct GlobalField<E> {
    pub id:         Option<Id>,
    pub exports:    Vec<String>,
    pub globaltype: GlobalType,
    pub init:       E,
    pub location:   Location,
}

impl<E: Debug> fmt::Debug for GlobalField<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.id {
            Some(id) => write!(f, "(global {:?} {:?} {:?})", id, self.globaltype, self.init),
            None => write!(f, "(global {:?} {:?})", self.globaltype, self.init),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportDesc<R: ResolvedState> {
    Func(TypeUse<R>),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

#[derive(PartialEq)]
pub struct ImportField<R: ResolvedState> {
    pub id:       Option<Id>,
    pub modname:  String,
    pub name:     String,
    pub exports:  Vec<String>,
    pub desc:     ImportDesc<R>,
    pub location: Location,
}

impl<R: ResolvedState> fmt::Debug for ImportField<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(import")?;
        if let Some(id) = &self.id {
            write!(f, " {}", id)?;
        }
        write!(
            f,
            " \"{}\" \"{}\" {:?})",
            self.modname, self.name, self.desc
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum ExportDesc<R: ResolvedState> {
    Func(Index<R, FuncIndex>),
    Table(Index<R, TableIndex>),
    Mem(Index<R, MemoryIndex>),
    Global(Index<R, GlobalIndex>),
}

// export := (export <name> <exportdesc>)
#[derive(PartialEq)]
pub struct ExportField<R: ResolvedState> {
    pub name:       String,
    pub exportdesc: ExportDesc<R>,
    pub location:   Location,
}

impl<R: ResolvedState> fmt::Debug for ExportField<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(export {} {:?})", self.name, self.exportdesc)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UncompiledExpr<R: ResolvedState> {
    pub instr: Vec<Instruction<R>>,
}
#[derive(Debug, Default, PartialEq)]
pub struct CompiledExpr {
    pub instr: Box<[u8]>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Opcode {
    Normal(u8),
    // 0xFC-prefixed instructions
    Extended(u8),
    // 0xFD-prefix instructions
    Simd(u8),
}

#[derive(Clone, PartialEq)]
pub struct Instruction<R: ResolvedState> {
    pub name:     Id,
    pub opcode:   Opcode,
    pub operands: Operands<R>,
    pub location: Location,
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal(o) => write!(f, "{:#x}", o),
            Self::Extended(o) => write!(f, "0xFC {:#x}", o),
            Self::Simd(o) => write!(f, "0xFD {:#x}", o),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Continuation {
    Start,
    End,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Operands<R: ResolvedState> {
    None,
    CallIndirect(Index<R, TableIndex>, TypeUse<R>),
    Block(Option<Id>, TypeUse<R>, UncompiledExpr<R>, Continuation),
    If(Option<Id>, TypeUse<R>, UncompiledExpr<R>, UncompiledExpr<R>),
    BrTable(Vec<Index<R, LabelIndex>>, Index<R, LabelIndex>),
    SelectT(Vec<FResult>),
    FuncIndex(Index<R, FuncIndex>),
    TableIndex(Index<R, TableIndex>),
    GlobalIndex(Index<R, GlobalIndex>),
    ElemIndex(Index<R, ElemIndex>),
    DataIndex(Index<R, DataIndex>),
    LocalIndex(Index<R, LocalIndex>),
    LabelIndex(Index<R, LabelIndex>),
    MemoryIndex(Index<R, MemoryIndex>),
    Memargs(u32, u32),
    HeapType(RefType),
    TableInit(Index<R, TableIndex>, Index<R, ElemIndex>),
    TableCopy(Index<R, TableIndex>, Index<R, TableIndex>),
    I32(u32),
    I64(u64),
    F32(f32),
    F64(f64),
}

impl<R: ResolvedState> std::fmt::Display for Operands<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operands::Block(id, ft, e, cnt) => {
                writeln!(f, "{:?} {:?} {:?}", id, ft, cnt)?;
                writeln!(f, "  {:?}", e)?;
                write!(f, ")")
            }
            Operands::If(id, ft, th, el) => {
                writeln!(f, "{:?} {:?}", id, ft)?;
                writeln!(f, "  (then  {:?})", th)?;
                writeln!(f, "  (else {:?})", el)?;
                write!(f, ")")
            }
            o => write!(f, "{:?}", o),
        }
    }
}

impl<R: ResolvedState> std::fmt::Debug for Instruction<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}({}) {}) ({}:{})",
            self.name, self.opcode, self.operands, self.location.line, self.location.pos
        )
    }
}

// start := (start <funcidx>)
#[derive(Debug, PartialEq)]
pub struct StartField<R: ResolvedState> {
    pub idx:      Index<R, FuncIndex>,
    pub location: Location,
}

#[derive(Debug, Default, PartialEq)]
pub struct TableUse<R: ResolvedState> {
    pub tableidx: Index<R, TableIndex>,
}

#[derive(Debug, PartialEq)]
pub struct TablePosition<R: ResolvedState, E> {
    pub tableuse: TableUse<R>,
    pub offset:   E,
}

// ElemList may be exprs, or func indices (unresolved)
#[derive(Debug, PartialEq)]
pub struct ElemList<E> {
    pub reftype: RefType,
    pub items:   Vec<E>,
}

impl<E> ElemList<E> {
    pub fn func(items: Vec<E>) -> Self {
        ElemList {
            reftype: RefType::Func,
            items,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ModeEntry<R: ResolvedState, E> {
    Passive,
    Active(TablePosition<R, E>),
    Declarative,
}

// elem := (elem <id>? <elemlist>)
//       | (elem <id>? <tableuse> (offset <expr>) <elemlist>)
//       | (elem <id>? declare <elemlist>)
#[derive(Debug, PartialEq)]
pub struct ElemField<R: ResolvedState, E> {
    pub id:       Option<Id>,
    pub mode:     ModeEntry<R, E>,
    pub elemlist: ElemList<E>,
    pub location: Location,
}

#[derive(Debug, PartialEq)]
pub struct DataInit<R: ResolvedState, E> {
    pub memidx: Index<R, MemoryIndex>,
    pub offset: E,
}

// data := (data id? <datastring>)
//       | (data id? <memuse> (offset <expr>) <datastring>)
// datastring := bytestring
// memuse := (memory <memidx>)
#[derive(Debug, PartialEq)]
pub struct DataField<R: ResolvedState, E> {
    pub id:       Option<Id>,
    pub data:     Box<[u8]>,
    pub init:     Option<DataInit<R, E>>,
    pub location: Location,
}
