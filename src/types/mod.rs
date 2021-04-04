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
