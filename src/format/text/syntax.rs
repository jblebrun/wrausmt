//! The syntax elements related to parsing a module.
//!
//! [Spec]: https://webassembly.github.io/spec/core/text/modules.html#modules

use crate::types::{GlobalType, ValueType, TableType, MemType, RefType};
use std::{collections::HashMap, fmt::{self, Debug}, marker::PhantomData};

/// ResolvedState is used to track whether or not the symbolic indices in the module have been
/// resolved into the proper numeric values. This needs to happen in a second pass after the
/// initial parse, since index usage may occur before the index has been defined.
///
pub trait ResolvedState : Debug {}

/// A module parameterized by the [Resolved] type will have undergone index resolution, and should
/// be safe to compile further.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Resolved {}
impl ResolvedState for Resolved {}

/// A module parameterized by the [Resolved] type will have undergone index resolution, and must be
/// compiled before it can be used by the runtime.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Unresolved {}
impl ResolvedState for Unresolved {}

pub trait IndexSpace {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FuncIndex {}
impl IndexSpace for FuncIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TypeIndex {}
impl IndexSpace for TypeIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableIndex {}
impl IndexSpace for TableIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GlobalIndex {}
impl IndexSpace for GlobalIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MemoryIndex {}
impl IndexSpace for MemoryIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataIndex {}
impl IndexSpace for DataIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElemIndex {}
impl IndexSpace for ElemIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LocalIndex {}
impl IndexSpace for LocalIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LabelIndex {}
impl IndexSpace for LabelIndex {}

/// Represents one index usage point. It may be named ($id) or numeric. [Spec]
///
/// An Index<Resolved> will have the correct numeric value associated. Index<Unresolved> may
/// contain a numeric value if one was parsed, but may also contain only a string name and a
/// default zero value.
///
/// [Spec]: https://webassembly.github.io/spec/core/text/modules.html#indices
#[derive(Clone, Default, PartialEq)]
pub struct Index<R:ResolvedState, S:IndexSpace> {
    pub name: String,
    pub value: u32,
    resolvedmarker: PhantomData<R>,
    indexmarker: PhantomData<S>,
}

impl <R:ResolvedState, S:IndexSpace> Index<R, S> {
    pub fn named(name: String, value: u32) -> Self {
        Index {
            name, 
            value,
            resolvedmarker: PhantomData::default(),
            indexmarker: PhantomData::default()
        }
    }
    pub fn unnamed(value: u32) -> Self {
        Index::named("".to_owned(), value)
    }

    pub fn resolved(self, value: u32) -> Index<Resolved, S> {
        Index {
            name: self.name,
            value, 
            resolvedmarker: PhantomData{},
            indexmarker: PhantomData{}
        }
    }
}

impl <R:ResolvedState, S:IndexSpace> fmt::Debug for Index<R, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.name, self.value)
    }
}

#[derive(Default, PartialEq)]
/// A parsed text format module. [Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/text/modules.html#modules
pub struct Module<R:ResolvedState> {
    pub id: Option<String>,
    pub types: Vec<TypeField>,
    pub funcs: Vec<FuncField<R>>,
    pub tables: Vec<TableField<R>>,
    pub memories: Vec<MemoryField>,
    pub imports: Vec<ImportField<R>>,
    pub exports: Vec<ExportField<R>>,
    pub globals: Vec<GlobalField<R>>,
    pub start: Option<StartField<R>>,
    pub elems: Vec<ElemField<R>>,
    pub data: Vec<DataField<R>>,
    pub identifiers: ModuleIdentifiers
}

#[derive(Default, Debug, PartialEq)]
pub struct ModuleIdentifiers {
    pub typeindices: HashMap<String, u32>,
    pub funcindices: HashMap<String, u32>,
    pub tableindices: HashMap<String, u32>,
    pub memindices: HashMap<String, u32>,
    pub globalindices: HashMap<String, u32>,
    pub elemindices: HashMap<String, u32>,
    pub dataindices: HashMap<String, u32>,
}

impl <I: ResolvedState> fmt::Debug for Module<I> {
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
        write!(f, "\n)")?;
        write!(f, ".IdentifierContext:")?;
        write!(f, "{:#?}", &self.identifiers)
    }
}

#[derive(Debug, PartialEq)]
pub enum Field<R:ResolvedState> {
    Type(TypeField),
    Func(FuncField<R>),
    Table(TableField<R>),
    Memory(MemoryField),
    Import(ImportField<R>),
    Export(ExportField<R>),
    Global(GlobalField<R>),
    Start(StartField<R>),
    Elem(ElemField<R>),
    Data(DataField<R>),
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
pub struct TypeUse<R:ResolvedState> {
    pub typeidx: Option<Index<R, TypeIndex>>,
    pub functiontype: FunctionType
}

impl <R:ResolvedState> TypeUse<R> {
    pub fn get_inline_def(&self) -> Option<FunctionType> {
        match self.typeidx {
            Some(_) => None,
            None => Some(self.functiontype.anonymous())
        }
    }
}

impl <R:ResolvedState> std::fmt::Debug for TypeUse<R> {
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
pub struct FuncField<R: ResolvedState> {
    pub id: Option<String>,
    pub exports: Vec<String>,
    pub typeuse: TypeUse<R>,
    pub locals: Vec<Local>,
    pub body: Expr<R>,
    pub localindices: HashMap<String, u32>,
}

impl <R:ResolvedState> std::fmt::Debug for FuncField<R> {
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
pub enum TableElems<R:ResolvedState> {
    Elem(ElemList<R>),
    Expr(Vec<Expr<R>>),
}

#[derive(Debug, PartialEq)]
// table :: = (table id? <tabletype>)
// Abbreviations:
// inline imports/exports
// inline elem
pub struct TableField<R:ResolvedState> {
    pub id: Option<String>,
    pub exports: Vec<String>,
    pub tabletype: TableType,
    pub elems: Option<TableElems<R>>
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
pub struct GlobalField<R:ResolvedState> {
    pub id: Option<String>,
    pub exports: Vec<String>,
    pub globaltype: GlobalType,
    pub init: Expr<R>,
}

impl <R:ResolvedState> fmt::Debug for GlobalField<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.id {
            Some(id) => write!(f, "(global {:?} {:?} {:?})", id, self.globaltype, self.init),
            None => write!(f, "(global {:?} {:?})", self.globaltype, self.init)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ImportDesc<R:ResolvedState> {
    Func(TypeUse<R>),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

#[derive(PartialEq)]
pub struct ImportField<R:ResolvedState> {
    pub modname: String,
    pub name: String,
    pub id: Option<String>,
    pub desc: ImportDesc<R>
}

impl <R:ResolvedState> fmt::Debug for ImportField<R> {
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
pub enum ExportDesc<R:ResolvedState> {
    Func(Index<R, FuncIndex>),
    Table(Index<R, TableIndex>),
    Mem(Index<R, MemoryIndex>),
    Global(Index<R, GlobalIndex>),
}

// export := (export <name> <exportdesc>)
#[derive(PartialEq)]
pub struct ExportField<R:ResolvedState> {
    pub name: String,
    pub exportdesc: ExportDesc<R>
}

impl <R:ResolvedState> fmt::Debug for ExportField<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(export {} {:?})", self.name, self.exportdesc)
    }
}

#[derive(Default, PartialEq)]
pub struct Expr<R:ResolvedState> {
    pub instr: Vec<Instruction<R>>
}

impl <R:ResolvedState> fmt::Debug for Expr<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in &self.instr {
            writeln!(f, "{:?}", i)?; 
        }
        Ok(())
    }
}

#[derive(PartialEq)]
pub struct Instruction<R:ResolvedState> {
    pub name: String,
    pub opcode: u8,
    pub operands: Operands<R>
}

#[derive(PartialEq, Debug)]
pub enum Operands<R:ResolvedState> {
    None,
    FuncIndex(Index<R, FuncIndex>),
    TableIndex(Index<R, TableIndex>),
    GlobalIndex(Index<R, GlobalIndex>),
    ElemIndex(Index<R, ElemIndex>),
    DataIndex(Index<R, DataIndex>),
    LocalIndex(Index<R, LocalIndex>),
    LabelIndex(Index<R, LabelIndex>),
    Memargs(u32, u32),
    I32(u32),
    I64(u64),
    F32(f32),
    F64(f64)
}

impl <R:ResolvedState> std::fmt::Debug for Instruction<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {:?})", self.name, self.operands)
    }
}

// start := (start <funcidx>)
#[derive(Debug, PartialEq)]
pub struct StartField<R:ResolvedState> {
    pub idx: Index<R, FuncIndex>
}

#[derive(Debug, PartialEq)]
pub struct TableUse<R:ResolvedState> {
    pub tableidx: Index<R, TableIndex>
}

#[derive(Debug, PartialEq)]
pub struct TablePosition<R:ResolvedState> {
    pub tableuse: TableUse<R>,
    pub offset: Expr<R>
}

#[derive(Debug, PartialEq)]
pub struct ElemList<R:ResolvedState> {
    pub reftype: RefType,
    pub items: Vec<Expr<R>>
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum ModeEntry<R:ResolvedState> {
    Passive,
    Active(TablePosition<R>),
    Declarative
}

// elem := (elem <id>? <elemlist>)
//       | (elem <id>? <tableuse> (offset <expr>) <elemlist>)
//       | (elem <id>? declare <elemlist>)
#[derive(Debug, PartialEq)]
pub struct ElemField<R:ResolvedState> {
    pub id: Option<String>,
    pub mode: ModeEntry<R>,
    pub elemlist: ElemList<R>,
}

#[derive(Debug, PartialEq)]
pub struct DataInit<R:ResolvedState> {
    pub memidx: Index<R, DataIndex>,
    pub offset: Expr<R>
}

// data := (data id? <datastring>)
//       | (data id? <memuse> (offset <expr>) <datastring>)
// datastring := bytestring
// memuse := (memory <memidx>)
#[derive(Debug, PartialEq)]
pub struct DataField<R:ResolvedState> {
    pub id: Option<String>,
    pub data: Vec<u8>,
    pub init: Option<DataInit<R>>
}
