use crate::runtime::values::Ref;
use crate::types::RefType;

/// An element instance is the runtime representation of an element segment.
/// [Spec][Spec]
///
/// It holds a vector of references and their common type.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#element-instances
#[derive(Debug)]
pub struct ElemInstance {
    pub elemtype: RefType,
    pub elems: Box<[Ref]>,
}
