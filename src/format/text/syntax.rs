//! The syntax elements related to parsing a module.
//!
//! [Spec]: https://webassembly.github.io/spec/core/text/modules.html#modules

use crate::types::{GlobalType, ValueType, TableType, MemType, RefType};
use std::fmt::{self, Debug};

/// Represents one index usage point. It may be named ($id) or numeric. [Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/text/modules.html#indices
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Default, PartialEq)]
/// A parsed text format module. [Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/text/modules.html#modules
pub struct Module {
    pub id: Option<String>,
    pub types: Vec<TypeField>,
    pub funcs: Vec<FuncField>,
    pub tables: Vec<TableField>,
    pub memories: Vec<MemoryField>,
    pub imports: Vec<ImportField>,
    pub exports: Vec<ExportField>,
    pub globals: Vec<GlobalField>,
    pub start: Option<StartField>,
    pub elems: Vec<ElemField>,
    pub data: Vec<DataField>,
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(module")?;
        macro_rules! print_all {
            ( $v:expr ) => {
                for i in $v {
                    write!(f, "\n{:?}", i)?;
                }
            }
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

        write!(f, ")")
    }
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

#[derive(PartialEq, Clone, Default)]
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

#[derive(PartialEq, Clone, Default)]
pub struct TypeUse {
    pub typeidx: Option<Index>,
    pub functiontype: FunctionType
}

impl TypeUse {
    pub fn get_inline_def(&self) -> Option<FunctionType> {
        match self.typeidx {
            Some(_) => None,
            None => Some(self.functiontype.anonymous())
        }
    }
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
#[derive(PartialEq, Clone)]
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
pub struct FuncField {
    pub id: Option<String>,
    pub exports: Vec<String>,
    pub typeuse: TypeUse,
    pub locals: Vec<Local>,
    pub body: Expr,
}

impl std::fmt::Debug for FuncField {
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
// table :: = (table id? <tabletype>)
// Abbreviations:
// inline imports/exports
// inline elem
pub struct TableField {
    pub id: Option<String>,
    pub exports: Vec<String>,
    pub tabletype: TableType,
    pub elems: Option<TableElems>
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
    pub init: Vec<u8>
}

// global := (global <id>? <globaltype> <expr>)
#[derive(PartialEq)]
pub struct GlobalField {
    pub id: Option<String>,
    pub exports: Vec<String>,
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
    Func(Index),
    Table(Index),
    Mem(Index),
    Global(Index),
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

#[derive(Default, PartialEq)]
pub struct Expr {
    pub instr: Vec<Instruction>
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in &self.instr {
            writeln!(f, "{:?}", i)?; 
        }
        Ok(())
    }
}

#[derive(PartialEq)]
pub struct Instruction {
    pub name: String,
    pub opcode: u8,
    pub operands: Operands
}

#[derive(PartialEq, Debug)]
pub enum Operands {
    None,
    Index(Index),
    Memargs(u32, u32),
    I32(u32),
    I64(u64),
    F32(f32),
    F64(f64)
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {:?})", self.name, self.operands)
    }
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
