use super::types::FunctionType;
use super::instructions::Inst;

#[derive(Debug)]
pub struct Module {
    pub types: Box<[FunctionType]>,
    pub funcs: Box<[Function]>
}

type TypeIndex = u32;

#[derive(Debug)]
pub struct Function {
    pub functype: TypeIndex,
    pub body: Box<[Inst]>
}
