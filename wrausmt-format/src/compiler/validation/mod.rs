//! Validation for instructions sequences as defined in [Spec].
//!
//! [Spec]: https://webassembly.github.io/spec/core/appendix/algorithm.html

use {
    wrausmt_common::true_or::TrueOr,
    wrausmt_runtime::{
        instructions::opcodes,
        syntax::{
            types::ValueType, Index, LabelIndex, LocalIndex, Module, Opcode, Resolved,
            UncompiledExpr,
        },
    },
};

mod ops;

#[derive(Debug)]
pub enum ValidationError {
    ValStackUnderflow,
    CtrlStackUnderflow,
    MemoryTooLarge,
    TableTooLarge,
    TypeMismatch {
        actual: ValidationType,
        expect: ValidationType,
    },
    UnusedValues,
    UnknownLocal(Index<Resolved, LocalIndex>),
    UnknownOpcode(Opcode),
    OpcodeMismatch,
    OperandsMismatch,
    LabelOutOfRange,
    BreakTypeMismatch,
}

/// How to treat Validator issues.
#[derive(Debug, Default, Clone, Copy)]
pub enum ValidationMode {
    // Ignore completely (the program will possibly crash in undefined ways based on the warnings
    // you see.)
    Warn,
    // The instantiation will fail by returning an error to the compile call.
    #[default]
    Fail,
    // Use panic to abort the entire process if validation fails.
    Panic,
}
pub type Result<T> = std::result::Result<T, ValidationError>;

/// Type representations during validation.
///
/// [Spec]: https://webassembly.github.io/spec/core/appendix/algorithm.html#data-structures
#[derive(Debug, Default, PartialEq)]
pub enum ValidationType {
    #[default]
    Unknown,
    Value(ValueType),
}

#[derive(Debug, PartialEq)]
pub struct CtrlFrame {
    opcode:      Opcode,
    start_types: Vec<ValueType>,
    end_types:   Vec<ValueType>,
    height:      usize,
    unreachable: bool,
}

/// The Validation context and implementation.
///
/// [Spec]: https://webassembly.github.io/spec/core/appendix/algorithm.html
pub struct Validation<'a> {
    pub mode: ValidationMode,

    pub module: &'a Module<Resolved, UncompiledExpr<Resolved>>,

    // Func
    pub localtypes: Vec<ValueType>,

    #[allow(dead_code)]
    val_stack:  Vec<ValueType>,
    ctrl_stack: Vec<CtrlFrame>,
}

impl<'a> Validation<'a> {
    pub fn new(
        mode: ValidationMode,
        module: &'a Module<Resolved, UncompiledExpr<Resolved>>,
        localtypes: Vec<ValueType>,
        resulttypes: Vec<ValueType>,
    ) -> Validation<'a> {
        let mut val = Validation {
            mode,
            module,
            localtypes,
            val_stack: Vec::default(),
            ctrl_stack: Vec::default(),
        };

        val.push_ctrl(opcodes::BLOCK, Vec::new(), resulttypes);
        val
    }

    fn push_val(&mut self, v: ValueType) {
        self.val_stack.push(v);
    }

    fn push_vals(&mut self, vs: &[ValueType]) {
        for v in vs.iter().rev() {
            self.val_stack.push(*v);
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
    fn pop_val(&mut self) -> Result<ValidationType> {
        let ctrl = self
            .ctrl_stack
            .last()
            .ok_or(ValidationError::CtrlStackUnderflow)?;
        if self.val_stack.len() == ctrl.height {
            return if ctrl.unreachable {
                Ok(ValidationType::Unknown)
            } else {
                Err(ValidationError::ValStackUnderflow)
            };
        }
        let val = self
            .val_stack
            .pop()
            .ok_or(ValidationError::ValStackUnderflow)?;
        Ok(ValidationType::Value(val))
    }

    fn pop_expect(&mut self, expect: ValueType) -> Result<ValidationType> {
        let actual = self.pop_val()?;
        let expect = ValidationType::Value(expect);
        match (actual, expect) {
            (ValidationType::Unknown, expect) => Ok(expect),
            (actual, ValidationType::Unknown) => Ok(actual),
            (actual, expect) => {
                if actual == expect {
                    Ok(actual)
                } else {
                    Err(ValidationError::TypeMismatch { actual, expect })
                }
            }
        }
    }

    fn pop_vals(&mut self, vs: &[ValueType]) -> Result<Vec<ValidationType>> {
        let mut result: Vec<ValidationType> = vec![];
        for v in vs.iter().rev() {
            result.push(self.pop_expect(*v)?);
        }
        Ok(result)
    }

    #[allow(dead_code)]
    fn push_ctrl(
        &mut self,
        opcode: Opcode,
        start_types: Vec<ValueType>,
        end_types: Vec<ValueType>,
    ) {
        self.push_vals(&start_types);
        let frame = CtrlFrame {
            opcode,
            start_types,
            end_types,
            height: self.val_stack.len(),
            unreachable: false,
        };
        self.ctrl_stack.push(frame)
    }

    #[allow(dead_code)]
    fn pop_ctrl(&mut self) -> Result<CtrlFrame> {
        let frame = self
            .ctrl_stack
            .last()
            .ok_or(ValidationError::CtrlStackUnderflow)?;
        let end_types = frame.end_types.clone();
        let cur_height = frame.height;
        self.pop_vals(&end_types)?;
        (self.val_stack.len() == cur_height).true_or(ValidationError::UnusedValues)?;
        let frame = self.ctrl_stack.pop().unwrap();
        Ok(frame)
    }

    fn label_types(&self, label: &Index<Resolved, LabelIndex>) -> Result<Vec<ValueType>> {
        let frame = self
            .ctrl_stack
            .get(self.ctrl_stack.len() - 1 - label.value() as usize)
            .ok_or(ValidationError::LabelOutOfRange)?;
        Ok(if frame.opcode == opcodes::LOOP {
            frame.start_types.clone()
        } else {
            frame.end_types.clone()
        })
    }

    fn unreachable(&mut self) -> Result<()> {
        let frame = self
            .ctrl_stack
            .last_mut()
            .ok_or(ValidationError::CtrlStackUnderflow)?;
        self.val_stack.truncate(frame.height);
        frame.unreachable = true;
        Ok(())
    }
}
