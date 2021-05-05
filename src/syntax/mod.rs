//! The syntax elements related to parsing a module.
//!
//! [Spec]: https://webassembly.github.io/spec/core/text/modules.html#modules

mod indices;

use crate::types::{GlobalType, MemType, RefType, TableType, ValueType};
pub use indices::{
    DataIndex, ElemIndex, FuncIndex, GlobalIndex, IndexSpace, LabelIndex, LocalIndex, MemoryIndex,
    TableIndex, TypeIndex,
};
pub use indices::{Resolved, ResolvedState, Unresolved};
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    marker::PhantomData,
};

/// Represents one index usage point. It may be named ($id) or numeric. [Spec]
///
/// An Index<Resolved> will have the correct numeric value associated. Index<Unresolved> may
/// contain a numeric value if one was parsed, but may also contain only a string name and a
/// default zero value.
///
/// [Spec]: https://webassembly.github.io/spec/core/text/modules.html#indices
#[derive(Clone, Default, PartialEq)]
pub struct Index<R: ResolvedState, S: IndexSpace> {
    name: String,
    value: u32,
    resolvedmarker: PhantomData<R>,
    indexmarker: PhantomData<S>,
}

impl<S: IndexSpace> From<Index<Resolved, S>> for u32 {
    fn from(idx: Index<Resolved, S>) -> u32 {
        idx.value()
    }
}

impl<R: ResolvedState, S: IndexSpace> Index<R, S> {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn named(name: String, value: u32) -> Self {
        Index {
            name,
            value,
            resolvedmarker: PhantomData::default(),
            indexmarker: PhantomData::default(),
        }
    }
    pub fn unnamed(value: u32) -> Self {
        Index::named("".to_owned(), value)
    }

    pub fn resolved(self, value: u32) -> Index<Resolved, S> {
        Index {
            name: self.name,
            value,
            resolvedmarker: PhantomData {},
            indexmarker: PhantomData {},
        }
    }
}

impl<R: ResolvedState, S: IndexSpace> fmt::Debug for Index<R, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.name, self.value)
    }
}

#[derive(Default, PartialEq)]
/// A parsed text format module. [Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/text/modules.html#modules
pub struct Module<R: ResolvedState> {
    pub id: Option<String>,
    pub types: Vec<TypeField>,
    pub funcs: Vec<FuncField<R>>,
    pub tables: Vec<TableField>,
    pub memories: Vec<MemoryField>,
    pub imports: Vec<ImportField<R>>,
    pub exports: Vec<ExportField<R>>,
    pub globals: Vec<GlobalField<R>>,
    pub start: Option<StartField<R>>,
    pub elems: Vec<ElemField<R>>,
    pub data: Vec<DataField<R>>,
}

impl<I: ResolvedState> fmt::Debug for Module<I> {
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
    pub params: Vec<FParam>,
    pub results: Vec<FResult>,
}

impl FunctionType {
    pub fn anonymous(&self) -> FunctionType {
        FunctionType {
            params: self
                .params
                .iter()
                .map(|p| FParam {
                    id: None,
                    valuetype: p.valuetype,
                })
                .collect(),
            results: self.results.clone(),
        }
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

/// A [Resolved] TypeUse has not just its index name resolved, but also provides a guarantee
/// that the index value stored corresponds to a type use in this module.
#[derive(PartialEq, Clone, Default)]
pub struct TypeUse<R: ResolvedState> {
    pub typeidx: Option<Index<R, TypeIndex>>,
    pub functiontype: FunctionType,
}

impl<R: ResolvedState> TypeUse<R> {
    pub fn get_inline_def(&self) -> Option<FunctionType> {
        match self.typeidx {
            Some(_) => None,
            None => Some(self.functiontype.anonymous()),
        }
    }
}

impl TypeUse<Resolved> {
    pub fn index_value(&self) -> u32 {
        match &self.typeidx {
            Some(idx) => idx.value(),
            None => panic!("improperly resolved typeuse (compiler error)"),
        }
    }
}

impl<R: ResolvedState> std::fmt::Debug for TypeUse<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(typeidx) = &self.typeidx {
            write!(f, "(type {:?})", typeidx)?;
        }

        write!(f, " {:?}", self.functiontype)
    }
}

// param := (param id? valtype)
#[derive(PartialEq, Clone)]
pub struct FParam {
    pub id: Option<String>,
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

// type := (type id? <functype>)
// functype := (func <param>* <result>*)
#[derive(PartialEq, Default)]
pub struct TypeField {
    pub id: Option<String>,
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
pub struct FuncField<R: ResolvedState> {
    pub id: Option<String>,
    pub exports: Vec<String>,
    pub typeuse: TypeUse<R>,
    pub locals: Vec<Local>,
    pub body: Expr<R>,
    pub localindices: HashMap<String, u32>,
}

impl<R: ResolvedState> std::fmt::Debug for FuncField<R> {
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
    pub id: Option<String>,
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
    pub id: Option<String>,
    pub exports: Vec<String>,
    pub tabletype: TableType,
}

// memory := (memory id? <memtype>)
//
// Abbreviations:
// Inline import/export
// Inline data segments
#[derive(Debug, PartialEq)]
pub struct MemoryField {
    pub id: Option<String>,
    pub exports: Vec<String>,
    pub memtype: MemType,
    pub init: Vec<u8>,
}

// global := (global <id>? <globaltype> <expr>)
#[derive(PartialEq)]
pub struct GlobalField<R: ResolvedState> {
    pub id: Option<String>,
    pub exports: Vec<String>,
    pub globaltype: GlobalType,
    pub init: Expr<R>,
}

impl<R: ResolvedState> fmt::Debug for GlobalField<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.id {
            Some(id) => write!(f, "(global {:?} {:?} {:?})", id, self.globaltype, self.init),
            None => write!(f, "(global {:?} {:?})", self.globaltype, self.init),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ImportDesc<R: ResolvedState> {
    Func(TypeUse<R>),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

#[derive(PartialEq)]
pub struct ImportField<R: ResolvedState> {
    pub modname: String,
    pub name: String,
    pub id: Option<String>,
    pub desc: ImportDesc<R>,
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
    pub name: String,
    pub exportdesc: ExportDesc<R>,
}

impl<R: ResolvedState> fmt::Debug for ExportField<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(export {} {:?})", self.name, self.exportdesc)
    }
}

#[derive(Default, PartialEq)]
pub struct Expr<R: ResolvedState> {
    pub instr: Vec<Instruction<R>>,
}

impl<R: ResolvedState> fmt::Debug for Expr<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in &self.instr {
            writeln!(f, "{:?}", i)?;
        }
        Ok(())
    }
}

#[derive(PartialEq)]
pub struct Instruction<R: ResolvedState> {
    pub name: String,
    pub opcode: u8,
    pub operands: Operands<R>,
}

impl<R: ResolvedState> Instruction<R> {
    pub fn reffunc(idx: Index<R, FuncIndex>) -> Self {
        Self {
            name: "ref.func".to_owned(),
            opcode: 0xD2,
            operands: Operands::FuncIndex(idx),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Continuation {
    Start,
    End,
}

#[derive(PartialEq, Debug)]
pub enum Operands<R: ResolvedState> {
    None,
    CallIndirect(Index<R, TableIndex>, TypeUse<R>),
    Block(Option<String>, TypeUse<R>, Expr<R>, Continuation),
    If(Option<String>, TypeUse<R>, Expr<R>, Expr<R>),
    BrTable(Vec<Index<R, LabelIndex>>),
    FuncIndex(Index<R, FuncIndex>),
    TableIndex(Index<R, TableIndex>),
    GlobalIndex(Index<R, GlobalIndex>),
    ElemIndex(Index<R, ElemIndex>),
    DataIndex(Index<R, DataIndex>),
    LocalIndex(Index<R, LocalIndex>),
    LabelIndex(Index<R, LabelIndex>),
    Memargs(u32, u32),
    HeapType(RefType),
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
        write!(f, "({}({:#x}) {})", self.name, self.opcode, self.operands)
    }
}

// start := (start <funcidx>)
#[derive(Debug, PartialEq)]
pub struct StartField<R: ResolvedState> {
    pub idx: Index<R, FuncIndex>,
}

#[derive(Debug, Default, PartialEq)]
pub struct TableUse<R: ResolvedState> {
    pub tableidx: Index<R, TableIndex>,
}

#[derive(Debug, Default, PartialEq)]
pub struct TablePosition<R: ResolvedState> {
    pub tableuse: TableUse<R>,
    pub offset: Expr<R>,
}

// ElemList may be exprs, or func indices (unresolved)
#[derive(Debug, PartialEq)]
pub struct ElemList<R: ResolvedState> {
    pub reftype: RefType,
    pub items: Vec<Expr<R>>,
}

impl<R: ResolvedState> ElemList<R> {
    pub fn func(items: Vec<Expr<R>>) -> Self {
        ElemList {
            reftype: RefType::Func,
            items,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ModeEntry<R: ResolvedState> {
    Passive,
    Active(TablePosition<R>),
    Declarative,
}

// elem := (elem <id>? <elemlist>)
//       | (elem <id>? <tableuse> (offset <expr>) <elemlist>)
//       | (elem <id>? declare <elemlist>)
#[derive(Debug, PartialEq)]
pub struct ElemField<R: ResolvedState> {
    pub id: Option<String>,
    pub mode: ModeEntry<R>,
    pub elemlist: ElemList<R>,
}

#[derive(Debug, PartialEq)]
pub struct DataInit<R: ResolvedState> {
    pub memidx: Index<R, DataIndex>,
    pub offset: Expr<R>,
}

// data := (data id? <datastring>)
//       | (data id? <memuse> (offset <expr>) <datastring>)
// datastring := bytestring
// memuse := (memory <memidx>)
#[derive(Debug, PartialEq)]
pub struct DataField<R: ResolvedState> {
    pub id: Option<String>,
    pub data: Vec<u8>,
    pub init: Option<DataInit<R>>,
}
