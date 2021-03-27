use super::types::FunctionType;
use super::instructions::Inst;

#[derive(Debug)]
pub struct Module {
    pub types: Box<[FunctionType]>,
    pub funcs: Box<[Function]>,
    pub exports: Box<[Export]>
}

pub type TypeIndex = u32;
pub type FuncIdx = u32;


#[derive(Debug)]
pub struct Export {
    pub name: String,
    pub idx: FuncIdx
}

#[derive(Debug)]
pub struct Function {
    pub functype: TypeIndex,
    pub body: Box<[Inst]>
}
