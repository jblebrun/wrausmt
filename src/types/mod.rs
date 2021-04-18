//! The representations of WASM types used by the various components of [wrausment][Spec].
//!
//! Various entities in WebAssembly are classified by types. Types are checked during validation,
//! instantiation, and possibly execution.
//!
//! [Spec]: https://webassembly.github.io/spec/core/syntax/types.html
use std::boxed::Box;

/// Number types classify numeric values. [Spec][Spec]
///
/// The types i32 and i64 classify 32 and 64 bit integers, respectively. Integers are not
/// inherently signed or unsigned, their interpretation is determined by individual operations. The
/// types f32 and f64 classify 32 and 64 bit floating-point data, respectively. They correspond to
/// the respective binary floating-point representations, also known as single and double
/// precision, as defined by the IEEE 754-2019 standard (Section 3.3).
///
/// Number types are transparent, meaning that their bit patterns can be observed. Values of number
/// type can be stored in memories.
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/types.html#number-types
#[derive(Debug, Copy, Clone)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

/// Reference types classify first-class references to objects in the runtime store. [Spec][Spec]
///
/// The type funcref denotes the infinite union of all references to functions, regardless of their
/// function types.  
///
/// The type externref denotes the infinite union of all references to objects
/// owned by the embedder and that can be passed into WebAssembly under this type.
///
/// Reference types are opaque, meaning that neither their size nor their bit pattern can be
/// observed. Values of reference type can be stored in tables.
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/types.html#reference-types
#[derive(Debug, Copy, Clone)]
pub enum RefType {
    Func,
    Extern,
}

/// Value types classify the individual values that WebAssembly code can compute with and the
/// values that a variable accepts. [Spec][Spec]
///
/// They are either [number types][NumType] or [reference types][RefType].
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/types.html#value-types
#[derive(Debug, Copy, Clone)]
pub enum ValueType {
    Num(NumType),
    Ref(RefType),
}

/// Result types classify the result of executing instructions or functions, which is a sequence of
/// values, written with brackets. [Spec][Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/types.html#result-types
pub type ResultType = [ValueType];

/// In the spec, param values use the [ResultType] type. To help with clarity, we define the
/// [ParamsType] alias.
pub type ParamsType = ResultType;

/// Function types classify the signature of functions, mapping a vector of parameters to a vector
/// of results. They are also used to classify the inputs and outputs of instructions. [Spec][Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/types.html#funcion-types
#[derive(Debug, Clone)]
pub struct FunctionType {
    pub params: Box<ParamsType>,
    pub result: Box<ResultType>,
}

/// Limits classify the size range of resizeable storage associated with [memory types][MemType]
/// and [table types][TableType]. [Spec][Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/types.html#limits
#[derive(Debug, Default, Clone)]
pub struct Limits {
    pub lower: u32,
    pub upper: Option<u32>,
}

/// Memory types classify linear memories and their size range. [Spec][Spec]
///
/// The limits constrain the minimum and optionally the maximum size of a memory. The limits are
/// given in units of page size.
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/types.html#memory-types
#[derive(Debug, Default, Clone)]
pub struct MemType {
    pub limits: Limits,
}

/// Table types classify tables over elements of reference type within a size range. [Spec][Spec]
///
/// Like memories, tables are constrained by limits for their minimum and optionally maximum size.
/// The limits are given in numbers of entries.
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/types.html#table-types
#[derive(Debug, Clone)]
pub struct TableType {
    pub limits: Limits,

    /// The [RefType] contained by this table type.
    pub reftype: RefType,
}

/// Global types classify global variables, which hold a value and can either be mutable or
/// immutable. [Spec][Spec]
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/types.html#global-types
#[derive(Debug, Clone)]
pub struct GlobalType {
    /// If true, the type refers to a mutable global value.
    pub mutable: bool,
    pub valtype: ValueType,
}

impl From<NumType> for ValueType {
    fn from(nt: NumType) -> ValueType {
        ValueType::Num(nt)
    }
}

impl From<RefType> for ValueType {
    fn from(rt: RefType) -> ValueType {
        ValueType::Ref(rt)
    }
}
