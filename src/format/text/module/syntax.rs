//! The syntax elements related to parsing a module.
//!
//! [Spec]: https://webassembly.github.io/spec/core/text/modules.html#modules

use crate::types::{GlobalType, ValueType, TableType, MemType, RefType};

#[derive(Debug, PartialEq)]
/// Represents one index usage point. It may be named ($id) or numeric. [Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/text/modules.html#indices
pub enum Index {
    Numeric(u32),
    Named(String)
}

#[derive(Debug, PartialEq)]
/// A parsed text format module. [Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/text/modules.html#modules
pub struct Module {
    pub id: Option<String>,
    pub fields: Vec<Field>
}

#[derive(Debug, PartialEq)]
pub enum Field {
    Type(TypeField),
    Func(FuncField),
    Table(TableField),
    Memory(MemoryField),
    Import(ImportField),
    Export(ExportField),
    Global(GlobalField),
    Start(StartField),
    Elem(ElemField),
    Data(DataField),
}

#[derive(Debug, PartialEq, Default)]
pub struct TypeUse {
    pub typeidx: Option<Index>,
    pub params: Vec<FParam>,
    pub results: Vec<FResult>
}

impl Default for Index {
    fn default() -> Self { Self::Numeric(0) }
}
// param := (param id? valtype)
#[derive(Debug, PartialEq)]
pub struct FParam {
    pub id: Option<String>,
    pub valuetype: ValueType,
}

// result := (result valtype)
#[derive(Debug, PartialEq)]
pub struct FResult {
    pub valuetype: ValueType,
}

// type := (type id? <functype>)
// functype := (func <param>* <result>*)
#[derive(Debug, PartialEq, Default)]
pub struct TypeField {
    pub id: Option<String>,
    pub params: Vec<FParam>,
    pub results: Vec<FResult>
}

// func := (func id? <typeuse> <local>* <instr>*)
// instr := sequence of instr, or folded expressions
//
// Abbreviations:
// func := (func id? (export  <name>)*  ...)
// func := (func id? (import <modname> <name>) <typeuse>)
#[derive(Debug, PartialEq)]
pub struct FuncField {
    pub id: Option<String>,
    pub exports: Vec<String>,
    pub typeuse: TypeUse,
    pub contents: FuncContents
}

// local := (local id? <valtype>)
#[derive(Debug, PartialEq)]
pub struct Local {
    pub id: Option<String>,
    pub valtype: ValueType
}

// Function fields may define a new function, or they may be an inline import.
#[derive(Debug, PartialEq)]
pub enum FuncContents {
    Inline{locals: Vec<Local>, body: Expr},
    Import{modname: String, name: String}
}

impl Default for FuncContents {
    fn default() -> Self { 
        FuncContents::Inline{locals: vec![], body: Expr::default() } 
    }
}

#[derive(Debug, PartialEq)]
pub enum TableElems {
    Elem(ElemList),
    Expr(Vec<Expr>),
}

#[derive(Debug, PartialEq)]
// Table may either be an import, or declaring a new table,
// in which case the contents may include initializer element segments.
pub enum TableContents {
    Inline{elems: Option<TableElems>},
    Import(String),
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
    pub contents: TableContents
}

#[derive(Debug, PartialEq)]
pub enum MemoryContents {
    // standard
    Inline(MemType),
    // inline init
    Initialized(Vec<u8>),
    // inline import
    Import(String)
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
    pub contents: MemoryContents,
}

// global := (global <id>? <globaltype> <expr>)
#[derive(Debug, PartialEq)]
pub struct GlobalField {
    pub id: Option<String>,
    pub globaltype: GlobalType,
    pub init: Expr,
}

#[derive(Debug, PartialEq)]
pub enum ImportDesc {
    Func(TypeUse),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

impl Default for ImportDesc {
    fn default() -> Self {
        Self::Func(TypeUse::default())
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct ImportField {
    pub modname: String,
    pub name: String,
    pub id: Option<String>,
    pub desc: ImportDesc
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum ExportDesc {
    Func(TypeUse),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

// export := (export <name> <exportdesc>)
#[derive(Debug, PartialEq)]
pub struct ExportField {
    pub name: String,
    pub exportdesc: ExportDesc
}

#[derive(Debug, Default, PartialEq)]
pub struct Expr {
}

// start := (start <funcidx>)
#[derive(Debug, PartialEq)]
pub struct StartField {
    pub idx: Index
}

#[derive(Debug, PartialEq)]
pub struct TableUse {
    pub tableidx: Index
}

#[derive(Debug, PartialEq)]
pub struct TablePosition {
    pub tableuse: TableUse,
    pub offset: Expr
}

#[derive(Debug, PartialEq)]
pub struct ElemList {
    pub reftype: RefType,
    pub items: Vec<Expr>
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum ModeEntry {
    Passive,
    Active(TablePosition),
    Declarative
}

// elem := (elem <id>? <elemlist>)
//       | (elem <id>? <tableuse> (offset <expr>) <elemlist>)
//       | (elem <id>? declare <elemlist>)
#[derive(Debug, PartialEq)]
pub struct ElemField {
    pub id: Option<String>,
    pub mode: ModeEntry,
    pub elemlist: ElemList,
}

#[derive(Debug, PartialEq)]
pub struct DataInit {
    pub memidx: Index,
    pub offset: Expr
}

// data := (data id? <datastring>)
//       | (data id? <memuse> (offset <expr>) <datastring>)
// datastring := bytestring
// memuse := (memory <memidx>)
#[derive(Debug, PartialEq)]
pub struct DataField {
    pub id: Option<String>,
    pub data: Vec<u8>,
    pub init: Option<DataInit>
}
