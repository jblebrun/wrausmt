use {
    self::{
        ctrl_stack::{CtrlFrame, CtrlStack},
        val_stack::ValueStack,
    },
    super::ValidationType,
    crate::{compiler::validation::KindResult as Result, ValidationErrorKind},
    wrausmt_common::true_or::TrueOr,
    wrausmt_runtime::syntax::{types::ValueType, Index, LabelIndex, Opcode, Resolved},
};

pub struct Stacks {
    ctrl: CtrlStack,
    val:  ValueStack,
}

impl Stacks {
    pub fn new() -> Stacks {
        let ctrl = CtrlStack::default();
        let val = ValueStack::new();
        Stacks { ctrl, val }
    }

    pub fn push_val(&mut self, val: impl Into<ValidationType>) {
        self.val.push(val.into())
    }

    pub fn push_vals(&mut self, vals: &[ValueType]) {
        self.val.push_many(vals)
    }

    pub fn pop_any(&mut self) -> Result<ValidationType> {
        self.val.pop_val(self.ctrl.peek()?)
    }

    pub fn pop_val(&mut self, expect: ValueType) -> Result<ValidationType> {
        let actual = self.val.pop_val(self.ctrl.peek()?)?;
        let expect = ValidationType::Value(expect);
        match (actual, expect) {
            (ValidationType::Unknown, _) => Ok(ValidationType::Unknown),
            (actual, ValidationType::Unknown) => Ok(actual),
            (actual, expect) => {
                if actual == expect {
                    Ok(actual)
                } else {
                    Err(ValidationErrorKind::TypeMismatch { actual, expect })
                }
            }
        }
    }

    pub fn pop_vals(&mut self, vs: &[ValueType]) -> Result<Vec<ValidationType>> {
        let mut result: Vec<ValidationType> = vec![];
        for v in vs.iter().rev() {
            result.push(self.pop_val(*v)?);
        }
        result.reverse();
        Ok(result)
    }

    pub fn push_ctrl(
        &mut self,
        opcode: Opcode,
        start_types: Vec<ValueType>,
        end_types: Vec<ValueType>,
    ) {
        let frame = CtrlFrame {
            opcode,
            start_types,
            end_types,
            height: self.val.len(),
            unreachable: false,
        };
        let frame = self.ctrl.push(frame);
        self.val.push_many(&frame.start_types);
    }

    pub fn pop_ctrl(&mut self) -> Result<CtrlFrame> {
        let frame = self.ctrl.peek()?;
        // TODO - remove clone?
        let end_types = frame.end_types.clone();
        let cur_height = frame.height;
        self.pop_vals(&end_types)?;
        (self.val.len() == cur_height).true_or(ValidationErrorKind::UnusedValues)?;
        self.ctrl.pop()
    }

    pub fn label_arity(&mut self, idx: &Index<Resolved, LabelIndex>) -> Result<usize> {
        Ok(self.ctrl.label_types(idx)?.len())
    }

    pub fn push_label_types(&mut self, idx: &Index<Resolved, LabelIndex>) -> Result<()> {
        let label_types = self.ctrl.label_types(idx)?;
        self.push_vals(&label_types);
        Ok(())
    }

    pub fn pop_label_types(
        &mut self,
        idx: &Index<Resolved, LabelIndex>,
    ) -> Result<Vec<ValidationType>> {
        // TODO - remove clone?
        let label_types = self.ctrl.label_types(idx)?.clone();
        self.pop_vals(&label_types)
    }

    pub fn pop_return_types(&mut self) -> Result<()> {
        let label_types = self.ctrl.return_types()?;
        self.pop_vals(&label_types).map(|_| ())
    }

    pub fn drop_val(&mut self) -> Result<()> {
        self.val.drop()
    }

    pub fn unreachable(&mut self) -> Result<()> {
        let new_height = self.ctrl.unreachable()?;
        self.val.truncate(new_height);
        Ok(())
    }
}

mod ctrl_stack;
mod val_stack;
