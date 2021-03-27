use std::boxed::Box;

#[derive(Debug)]
pub enum NumType { I32, I64, F32, F64 }

#[derive(Debug)]
pub enum RefType { Func, Extern }

#[derive(Debug)]
pub enum ValueType { Num(NumType), Ref(RefType) }

pub type ResultType = [ValueType];

#[derive(Debug)]
pub struct FunctionType {
    pub params: Box<ResultType>,
    pub result: Box<ResultType>
}

