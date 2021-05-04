use crate::runtime::values::Ref;
use crate::types::TableType;

/// A table instance is the runtime representation of a table. [Spec][Spec]
///
/// It records its type and holds a vector of reference values.
///
/// Table elements can be mutated through table instructions, the execution of an
/// active element segment, or by external means provided by the embedder.
///
/// It is an invariant of the semantics that all table elements have a type equal
/// to the element type of tabletype. It also is an invariant that the length of
/// the element vector never exceeds the maximum size of tabletype if present.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#table-instances
#[derive(Debug)]
pub struct TableInstance {
    pub tabletype: TableType,
    pub elem: Box<[Ref]>,
}

impl TableInstance {
    pub fn new(tabletype: TableType) -> TableInstance {
        let elem: Box<[Ref]> = std::iter::repeat(tabletype.reftype.default())
            .take(tabletype.limits.lower as usize)
            .collect();
        TableInstance { tabletype, elem }
    }
}
