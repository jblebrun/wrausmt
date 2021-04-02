use std::boxed::Box;

#[derive(Debug, Copy, Clone)]
pub enum NumType { I32, I64, F32, F64 }

#[derive(Debug, Copy, Clone)]
pub enum RefType { Func, Extern }

#[derive(Debug, Copy, Clone)]
pub enum ValueType { Num(NumType), Ref(RefType) }

pub type ResultType = [ValueType];

#[derive(Debug, Clone)]
pub struct FunctionType {
    pub params: Box<ResultType>,
    pub result: Box<ResultType>
}

