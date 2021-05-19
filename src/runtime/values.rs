//! The values that WebAssembly programs can manipulate. [Spec][Spec]
//!
//! WebAssembly computations manipulate values of either the four basic number types, i.e.,
//! integers and floating-point data of 32 or 64 bit width each, or of reference type.
//!
//! In most places of the semantics, values of different types can occur. In order to avoid ambiguities,
//! values are therefore represented with an abstract syntax that makes their type explicit. It is
//! convenient to reuse the same notation as for the const instructions and ref.null producing
//! them.
//!
//! References other than null are represented with additional administrative instructions. They
//! either are function references, pointing to a specific function address, or external references
//! pointing to an uninterpreted form of extern address that can be defined by the embedder to
//! represent its own objects.
//!
//! [Spec]: https://webassembly.github.io/spec/core/syntax/values.html#values

use crate::{
    impl_bug,
    runtime::error::RuntimeError,
    types::{NumType, RefType, ValueType},
};
use std::convert::TryFrom;

use super::store::addr;

/// A value that a WebAssembly program can manipulate. [Spec][Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/values.html#values
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Value {
    Num(Num),
    Ref(Ref),
}

/// A numeric value that a WebAssembly program can manipulate. [Spec][Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/values.html#values
#[derive(PartialEq, Clone, Copy)]
pub enum Num {
    I32(u32),
    I64(u64),
    F32(f32),
    F64(f64),
}

impl Num {
    pub fn numtype(&self) -> NumType {
        match self {
            Num::I32(_) => NumType::I32,
            Num::I64(_) => NumType::I64,
            Num::F32(_) => NumType::F32,
            Num::F64(_) => NumType::F64,
        }
    }
}

impl Ref {
    pub fn reftype(&self) -> RefType {
        match self {
            Ref::Func(_) => RefType::Func,
            Ref::Extern(_) => RefType::Extern,
            Ref::Null(rt) => *rt,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Ref::Null(_))
    }
}

impl Value {
    pub fn valtype(&self) -> ValueType {
        match self {
            Value::Num(n) => ValueType::Num(n.numtype()),
            Value::Ref(r) => ValueType::Ref(r.reftype()),
        }
    }

    pub fn is(&self, vt: ValueType) -> bool {
        self.valtype() == vt
    }

    pub fn as_num(&self) -> Option<Num> {
        match self {
            Value::Num(n) => Some(*n),
            _ => None,
        }
    }
}

impl std::fmt::Debug for Num {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Num::F32(i) => write!(f, "F32 {} ({:08x})", i, i.to_bits()),
            Num::F64(i) => write!(f, "F64 {} ({:016x})", i, i.to_bits()),
            Num::I32(i) => write!(f, "I32 {} ({:08x})", i, i),
            Num::I64(i) => write!(f, "I64 {} ({:016x})", i, i),
        }
    }
}

/// A reference value that a WebAssembly program can manipulate. [Spec][Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/values.html#values
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Ref {
    Func(addr::FuncAddr),
    Extern(u32),
    Null(RefType),
}

impl RefType {
    pub fn default(&self) -> Ref {
        match &self {
            RefType::Func => Ref::Null(RefType::Func),
            RefType::Extern => Ref::Null(RefType::Extern),
        }
    }
}

impl NumType {
    pub fn default(&self) -> Num {
        match &self {
            NumType::I32 => Num::I32(0),
            NumType::I64 => Num::I64(0),
            NumType::F32 => Num::F32(0f32),
            NumType::F64 => Num::F64(0f64),
        }
    }
}

impl ValueType {
    /// Provide the default/zero [Value] for the corresponding [ValueType]. [Spec][Spec]
    ///
    /// Each value type has an associated default value; it is the respective value 0 for number
    /// types and null for reference types.
    ///
    /// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#values
    pub fn default(&self) -> Value {
        match &self {
            ValueType::Num(n) => Value::Num(n.default()),
            ValueType::Ref(r) => Value::Ref(r.default()),
        }
    }
}

/// Here, we implement a number of convenience converters for the numeric types.
///
/// This makes it convenient to convert from the rust primitive type to either the value subtype
/// ([Num] or [Ref]), or the containing [Value] type that can hold either.
macro_rules! froms {
    ( $ty:ty, $sty:ty, $name:ident ) => {
        impl From<$ty> for Num {
            fn from(v: $ty) -> Num {
                Num::$name(v as $sty)
            }
        }

        impl From<$ty> for Value {
            fn from(v: $ty) -> Value {
                Value::Num(Num::$name(v as $sty))
            }
        }

        impl TryFrom<Value> for $ty {
            type Error = RuntimeError;

            fn try_from(val: Value) -> Result<$ty, Self::Error> {
                match val {
                    Value::Num(Num::$name(v)) => Ok(v as $ty),
                    _ => Err(impl_bug!(
                        "couldn't convert {:?} {}",
                        val,
                        stringify!($name)
                    )),
                }
            }
        }
    };
}

froms! { u32, u32, I32 }
froms! { u8, u32, I32 }
froms! { usize, u32, I32 }
froms! { u64, u64, I64 }
froms! { i32, u32, I32 }
froms! { i64, u64, I64 }
froms! { f32, f32, F32 }
froms! { f64, f64, F64 }

impl From<Num> for Value {
    fn from(n: Num) -> Value {
        Value::Num(n)
    }
}

impl From<Ref> for Value {
    fn from(r: Ref) -> Value {
        Value::Ref(r)
    }
}

impl TryFrom<Value> for Ref {
    type Error = RuntimeError;
    fn try_from(v: Value) -> Result<Ref, Self::Error> {
        match v {
            Value::Ref(r) => Ok(r),
            _ => Err(impl_bug!("{:?} is not a ref", v)),
        }
    }
}
