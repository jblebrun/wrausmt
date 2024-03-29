use {
    crate::compiler::validation::{KindResult as Result, ValidationErrorKind},
    wrausmt_common::true_or::TrueOr,
    wrausmt_runtime::{
        instructions::opcodes,
        syntax::{types::ValueType, Index, LabelIndex, Opcode, Resolved},
    },
};

/// One frame in the ctrl_stack, as defined by the spec.
///
/// [Spec]: https://webassembly.github.io/spec/core/appendix/algorithm.html#data-structures
#[derive(Debug, PartialEq)]
pub struct CtrlFrame {
    pub opcode:      Opcode,
    pub start_types: Vec<ValueType>,
    pub end_types:   Vec<ValueType>,
    pub height:      usize,
    pub unreachable: bool,
}

/// The ctrl stack for validation, a wrapper around the list of [`CtrlFrame`]
/// items exposing only well-defined mutation methods.
#[derive(Default, Debug)]
pub struct CtrlStack {
    frames: Vec<CtrlFrame>,
}

impl CtrlStack {
    pub fn peek(&self) -> Result<&CtrlFrame> {
        self.frames
            .last()
            .ok_or(ValidationErrorKind::CtrlStackUnderflow)
    }

    pub fn push(&mut self, frame: CtrlFrame) -> &CtrlFrame {
        self.frames.push(frame);
        self.frames.last().unwrap()
    }

    pub fn pop(&mut self) -> Result<CtrlFrame> {
        self.frames
            .pop()
            .ok_or(ValidationErrorKind::CtrlStackUnderflow)
    }

    pub fn label_types(&self, idx: &Index<Resolved, LabelIndex>) -> Result<Vec<ValueType>> {
        ((idx.value() as usize) < self.frames.len()).true_or(ValidationErrorKind::UnknownLabel)?;
        let frame = self
            .frames
            .get(self.frames.len() - 1 - idx.value() as usize)
            .ok_or(ValidationErrorKind::UnknownLabel)?;

        Ok(if frame.opcode == opcodes::LOOP {
            // TODO - return ref?
            frame.start_types.clone()
        } else {
            // TODO - return ref?
            frame.end_types.clone()
        })
    }

    pub fn return_types(&self) -> Result<Vec<ValueType>> {
        let idx = Index::unnamed((self.frames.len() - 1) as u32);
        self.label_types(&idx)
    }

    pub fn unreachable(&mut self) -> Result<usize> {
        let frame = self
            .frames
            .last_mut()
            .ok_or(ValidationErrorKind::CtrlStackUnderflow)?;

        frame.unreachable = true;
        Ok(frame.height)
    }
}
