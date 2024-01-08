use crate::{
    runtime::{
        error::{Result, RuntimeErrorKind, TrapKind},
        values::Ref,
    },
    syntax::types::TableType,
    validation::ValidationError,
};

/// A table instance is the runtime representation of a table. [Spec][Spec]
///
/// It records its type and holds a vector of reference values.
///
/// Table elements can be mutated through table instructions, the execution of
/// an active element segment, or by external means provided by the embedder.
///
/// It is an invariant of the semantics that all table elements have a type
/// equal to the element type of tabletype. It also is an invariant that the
/// length of the element vector never exceeds the maximum size of tabletype if
/// present.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#table-instances
#[derive(Debug)]
pub struct TableInstance {
    pub tabletype: TableType,
    pub elem:      Vec<Ref>,
}

impl TableInstance {
    pub fn new(tabletype: TableType) -> Result<TableInstance> {
        if tabletype.limits.lower > 0xFFFF {
            Err(RuntimeErrorKind::ValidationError(
                ValidationError::TableTooLarge,
            ))?;
        }
        let elem: Vec<Ref> = std::iter::repeat(tabletype.reftype.default())
            .take(tabletype.limits.lower as usize)
            .collect();
        Ok(TableInstance { tabletype, elem })
    }

    pub fn grow(&mut self, amt: u32, val: Ref) -> Option<u32> {
        let oldsize = self.elem.len();
        let newsize = self.elem.len() + amt as usize;
        if newsize > i32::MAX as usize {
            return None;
        }
        if matches!(self.tabletype.limits.upper, Some(upper) if newsize > upper as usize) {
            return None;
        }
        self.elem.resize(newsize, val);
        Some(oldsize as u32)
    }

    pub fn fill(&mut self, n: usize, val: Ref, i: usize) -> Result<()> {
        self.elem
            .get_mut(i..i + n)
            .ok_or(TrapKind::OutOfBoundsTableAccess(i, n))?
            .fill(val);
        Ok(())
    }
}
