use super::types::FunctionType;
use super::types::ValueType;
use super::types::RefType;
use super::instructions::Inst;

#[derive(Debug, Clone)]
pub struct Module {
    pub types: Box<[FunctionType]>,
    pub imports: Box<[Import]>,
    pub funcs: Box<[Function]>,
    pub exports: Box<[Export]>
}

pub type TypeIndex = u32;
pub type FuncIdx = u32;

#[derive(Debug, Clone)]
pub struct Limits {
    pub lower: u32,
    pub upper: Option<u32>
}

#[derive(Debug, Clone)]
pub struct TableType {
    pub limits: Limits,
    pub reftype: RefType
}

#[derive(Debug, Clone)]
pub struct MemType {
    pub limits: Limits,
}

#[derive(Debug, Clone)]
pub struct GlobalType {
    pub mutable: bool,
    pub valtype: ValueType
}

#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(TypeIndex),
    Table(TableType),
    Memory(MemType),
    Global(GlobalType)
}

#[derive(Debug, Clone)]
pub struct Import {
    pub module_name: String,
    pub name: String,
    pub desc: ImportDesc
}

#[derive(Debug, Clone)]
pub struct Export {
    pub name: String,
    pub idx: FuncIdx
}

#[derive(Debug, Clone)]
pub struct Function {
    pub functype: TypeIndex,
    pub locals: Box<[ValueType]>,
    pub body: Box<[Inst]>
}
