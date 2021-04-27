//! The syntax elements related to parsing a module.
//!
//! [Spec]: https://webassembly.github.io/spec/core/text/modules.html#modules

use crate::types::{GlobalType, ValueType, TableType, MemType, RefType};
use std::fmt;

#[derive(Debug, PartialEq)]
/// Represents one index usage point. It may be named ($id) or numeric. [Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/text/modules.html#indices
pub enum Index {
    Numeric(u32),
    Named(String)
}

impl Default for Index {
    fn default() -> Self { Self::Numeric(0) }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Numeric(val) => f.write_str(&val.to_string()),
            Self::Named(val) => f.write_str(val)
        }
    }
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

#[derive(PartialEq, Default)]
pub struct FunctionType {
    pub params: Vec<FParam>,
    pub results: Vec<FResult>
}

impl FunctionType {
    pub fn anonymous(&self) -> FunctionType {
        FunctionType {
            params: self.params.iter()
                .map(|p| FParam{id:None, valuetype:p.valuetype})
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

#[derive(PartialEq, Default)]
pub struct TypeUse {
    pub typeidx: Option<Index>,
    pub functiontype: FunctionType
}

impl std::fmt::Debug for TypeUse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(typeidx) = &self.typeidx {
            write!(f, "(type {})", typeidx)?;
        }

        write!(f, " {:?}", self.functiontype)
    }
}

// param := (param id? valtype)
#[derive(PartialEq)]
pub struct FParam {
    pub id: Option<String>,
    pub valuetype: ValueType,
}

impl std::fmt::Debug for FParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.id {
            Some(id) => write!(f, "(param {} {:?})", id, self.valuetype),
            None => write!(f, "(param {:?})", self.valuetype)
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
    pub functiontype: FunctionType
}

impl std::fmt::Debug for TypeField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = &self.id {
            write!(f, "(type {})", id)?;
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
pub struct FuncField {
    pub id: Option<String>,
    pub exports: Vec<String>,
    pub typeuse: TypeUse,
    pub locals: Vec<Local>,
    pub body: Expr,
}

impl std::fmt::Debug for FuncField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(func ")?;

        if let Some(id) = &self.id {
            write!(f, "{}", id)?;
        }

        for export in &self.exports {
            write!(f, "(export {})", export)?;
        }
        
        write!(f, "{:?}", self.typeuse)?;

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
    pub valtype: ValueType
}

impl std::fmt::Debug for Local {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.id {
            Some(id) => write!(f, "(local {} {:?})", id, self.valtype),
            None => write!(f, "(local {:?})", self.valtype)
        }
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
#[derive(PartialEq)]
pub struct GlobalField {
    pub id: Option<String>,
    pub globaltype: GlobalType,
    pub init: Expr,
}

impl fmt::Debug for GlobalField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.id {
            Some(id) => write!(f, "(global {} {:?} {:?})", id, self.globaltype, self.init),
            None => write!(f, "(global {:?} {:?})", self.globaltype, self.init)
        }
    }
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

#[derive(PartialEq, Default)]
pub struct ImportField {
    pub modname: String,
    pub name: String,
    pub id: Option<String>,
    pub desc: ImportDesc
}

impl fmt::Debug for ImportField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(import")?;
        if let Some(id) = &self.id {
            write!(f, " {}", id)?;
        }
        write!(f, " \"{}\" \"{}\" {:?})", self.modname, self.name, self.desc)
    }
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
#[derive(PartialEq)]
pub struct ExportField {
    pub name: String,
    pub exportdesc: ExportDesc
}

impl fmt::Debug for ExportField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(export {} {:?})", self.name, self.exportdesc)
    }
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
