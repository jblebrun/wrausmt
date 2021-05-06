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
    err,
    error::Error,
    types::{NumType, RefType, ValueType},
};
use std::convert::TryFrom;

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
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Num {
    I32(u32),
    I64(u64),
    F32(f32),
    F64(f64),
}

/// A reference value that a WebAssembly program can manipulate. [Spec][Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/values.html#values
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Ref {
    Func(u64),
    Extern(u64),
}

impl RefType {
    pub fn default(&self) -> Ref {
        match &self {
            RefType::Func => Ref::Func(0),
            RefType::Extern => Ref::Extern(0),
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
            type Error = Error;

            fn try_from(val: Value) -> Result<$ty, Self::Error> {
                match val {
                    Value::Num(Num::$name(v)) => Ok(v as $ty),
                    _ => err!("couldn't convert {:?} {}", val, stringify!($name)),
                }
            }
        }
    };
}

froms! { u32, u32, I32 }
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
