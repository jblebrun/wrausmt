use {
    super::ctrl_stack::CtrlFrame,
    crate::compiler::validation::{KindResult as Result, ValidationErrorKind, ValidationType},
    wrausmt_runtime::syntax::types::ValueType,
};

/// The value stack for validation, a wrapper around the list of
/// [`ValueType`] items exposing only well-defined mutation methods.
#[derive(Debug)]
pub struct ValueStack {
    values: Vec<ValueType>,
}

impl ValueStack {
    pub fn new() -> ValueStack {
        ValueStack { values: vec![] }
    }

    pub fn push(&mut self, v: ValueType) {
        self.values.push(v);
    }

    pub fn push_many(&mut self, vs: &[ValueType]) {
        for v in vs.iter().rev() {
            self.values.push(*v);
        }
    }

    /// Popping an operand value checks that the value stack does not underflow
    /// the current block and then removes one type. But first, a special case
    /// is handled where the block contains no known values, but has been marked
    /// as unreachable. That can occur after an unconditional branch, when the
    /// stack is typed polymorphically. In that case, an unknown type is
    /// returned.
    ///
    /// [See Spec](https://webassembly.github.io/spec/core/appendix/algorithm.html#data-structures)
    pub fn pop_val(&mut self, ctrl: &CtrlFrame) -> Result<ValidationType> {
        if self.len() == ctrl.height {
            return if ctrl.unreachable {
                Ok(ValidationType::Unknown)
            } else {
                Err(ValidationErrorKind::ValStackUnderflow)
            };
        }
        let val = self
            .values
            .pop()
            .ok_or(ValidationErrorKind::ValStackUnderflow)?;
        Ok(ValidationType::Value(val))
    }

    pub fn drop(&mut self) -> Result<()> {
        let _ = self
            .values
            .pop()
            .ok_or(ValidationErrorKind::ValStackUnderflow)?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn truncate(&mut self, height: usize) {
        self.values.truncate(height)
    }
}
