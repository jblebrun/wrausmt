//! Validation for instructions sequences as defined in [Spec].
//!
//! [Spec]: https://webassembly.github.io/spec/core/appendix/algorithm.html

use crate::types::{
    FunctionType, GlobalType, MemType, ParamsType, RefType, ResultType, TableType, ValueType,
};

#[derive(Debug)]
pub enum ValidationError {
    ValStackUnderflow,
    CtrlStackUnderflow,
    TypeMismatch(ValidationType, ValidationType),
    UnusedValues,
}

type Result<T> = std::result::Result<T, ValidationError>;

type FuncIndex = u32;

#[derive(Debug, Default)]
pub struct ValidationContext {
    // Module
    pub types: Vec<FunctionType>,
    pub funcs: Vec<FunctionType>,
    pub tables: Vec<TableType>,
    pub mems: Vec<MemType>,
    pub globals: Vec<GlobalType>,
    pub elems: Vec<RefType>,
    pub datas: Vec<()>,
    pub refs: Vec<FuncIndex>,

    // Function
    pub locals: Vec<ValueType>,

    // These may change throughout the sequence.
    pub labels: Vec<Box<ResultType>>,
    pub returns: Vec<Box<ResultType>>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationType {
    Unknown,
    Value(ValueType),
}

#[derive(Debug, PartialEq)]
pub struct CtrlFrame {
    opcode: u8,
    start_types: Box<ParamsType>,
    end_types: Box<ResultType>,
    height: usize,
    unreachable: bool,
}

pub struct Validation {
    val_stack: Vec<ValueType>,
    ctrl_stack: Vec<CtrlFrame>,
}

impl Validation {
    pub fn push_val(&mut self, v: ValueType) {
        self.val_stack.push(v);
    }

    pub fn push_vals(&mut self, vals: &[ValueType]) {
        for v in vals.iter().rev() {
            self.val_stack.push(*v);
        }
    }

    /// Popping an operand value checks that the value stack does not underflow the current block
    /// and then removes one type. But first, a special case is handled where the block contains no
    /// known values, but has been marked as unreachable. That can occur after an unconditional
    /// branch, when the stack is typed polymorphically. In that case, an unknown type is returned.
    ///
    /// [See Spec](https://webassembly.github.io/spec/core/appendix/algorithm.html#data-structures)
    pub fn pop_val(&mut self) -> Result<ValidationType> {
        let ctrl = self
            .ctrl_stack
            .last()
            .ok_or(ValidationError::CtrlStackUnderflow)?;
        if self.val_stack.len() == ctrl.height {
            if ctrl.unreachable {
                return Ok(ValidationType::Unknown);
            } else {
                return Err(ValidationError::ValStackUnderflow);
            }
        }
        let val = self
            .val_stack
            .pop()
            .ok_or(ValidationError::ValStackUnderflow)?;
        Ok(ValidationType::Value(val))
    }

    pub fn pop_expect(&mut self, expect: ValueType) -> Result<ValidationType> {
        let actual = self.pop_val()?;
        let expect = ValidationType::Value(expect);
        match (actual, expect) {
            (ValidationType::Unknown, expect) => Ok(expect),
            (actual, ValidationType::Unknown) => Ok(actual),
            (actual, expect) => {
                if actual == expect {
                    Ok(actual)
                } else {
                    Err(ValidationError::TypeMismatch(actual, expect))
                }
            }
        }
    }

    pub fn pop_vals(&mut self, expects: &[ValueType]) -> Result<Vec<ValidationType>> {
        let mut result: Vec<ValidationType> = vec![];
        for e in expects.iter().rev() {
            result.push(self.pop_expect(*e)?);
        }
        Ok(result)
    }

    pub fn push_ctrl(&mut self, opcode: u8, start_types: &ParamsType, end_types: &ResultType) {
        let frame = CtrlFrame {
            opcode,
            start_types: start_types.to_owned().into_boxed_slice(),
            end_types: end_types.to_owned().into_boxed_slice(),
            height: self.val_stack.len(),
            unreachable: false,
        };
        self.push_vals(start_types);
        self.ctrl_stack.push(frame)
    }

    pub fn pop_ctrl(&mut self) -> Result<CtrlFrame> {
        let frame = self
            .ctrl_stack
            .pop()
            .ok_or(ValidationError::CtrlStackUnderflow)?;
        let vals = self.pop_vals(&frame.end_types)?;
        if vals.len() != frame.height {
            return Err(ValidationError::UnusedValues);
        }
        Ok(frame)
    }

    pub fn unreachable(&mut self) -> Result<()> {
        let frame = self
            .ctrl_stack
            .last_mut()
            .ok_or(ValidationError::CtrlStackUnderflow)?;
        self.val_stack.truncate(frame.height);
        frame.unreachable = true;
        Ok(())
    }
}
