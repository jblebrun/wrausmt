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


#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Num {
    I32(u32),
    I64(u64),
    F32(f32),
    F64(f64),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Ref {
    Func(u64),
    Extern(u64),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Value {
    Num(Num),
    Ref(Ref)
}

impl ValueType {
    pub fn default(&self) -> Value {
        match &self {
            ValueType::Num(NumType::I32) => Value::Num(Num::I32(0)),
            ValueType::Num(NumType::I64) => Value::Num(Num::I64(0)),
            ValueType::Num(NumType::F32) => Value::Num(Num::F32(0f32)),
            ValueType::Num(NumType::F64) => Value::Num(Num::F64(0f64)),
            ValueType::Ref(RefType::Func) => Value::Ref(Ref::Func(0)),
            ValueType::Ref(RefType::Extern) => Value::Ref(Ref::Extern(0)),
        }
    }
}

macro_rules! froms {
    ( $ty:ty, $name:ident ) => {
        impl From<$ty> for Num {
            fn from(v: $ty) -> Num {
                Num::$name(v as $ty)
            }
        }

        impl From<$ty> for Value {
            fn from(v: $ty) -> Value {
                Value::Num(Num::$name(v as $ty))
            }
        }
    }
}

froms! { u32, I32 }
froms! { u64, I64 }
froms! { f32, F32 }
froms! { f64, F64 }

impl From<Num> for Value {
    fn from(n: Num) -> Value {
        Value::Num(n)
    }
}
