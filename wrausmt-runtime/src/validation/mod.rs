//! Validation for instructions sequences as defined in [Spec].
//!
//! [Spec]: https://webassembly.github.io/spec/core/appendix/algorithm.html

use {
    crate::syntax::types::{
        FunctionType, GlobalType, MemType, ParamsType, RefType, ResultType, TableType, ValueType,
    },
    wrausmt_common::true_or::TrueOr,
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
    UnknownOpcode,
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

type FuncIndex = u32;

/// The validation context for opcodes using indices, as described in [^Spec].
///
/// [Spec]: https://webassembly.github.io/spec/core/appendix/algorithm.html
#[derive(Debug, Default)]
pub struct ValidationContext {
    mode: ValidationMode,

    // Module
    pub types:   Vec<FunctionType>,
    pub funcs:   Vec<FunctionType>,
    pub tables:  Vec<TableType>,
    pub mems:    Vec<MemType>,
    pub globals: Vec<GlobalType>,
    pub elems:   Vec<RefType>,
    pub datas:   Vec<()>,
    pub refs:    Vec<FuncIndex>,

    // Function
    pub locals: Vec<ValueType>,

    // These may change throughout the sequence via control ops.
    pub labels:  Vec<Box<ResultType>>,
    pub returns: Vec<Box<ResultType>>,
}

impl ValidationContext {
    pub fn new(mode: ValidationMode) -> ValidationContext {
        ValidationContext {
            mode,
            ..Default::default()
        }
    }
}

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
    opcode:      u8,
    start_types: Box<ParamsType>,
    end_types:   Box<ResultType>,
    height:      usize,
    unreachable: bool,
}

impl Default for CtrlFrame {
    fn default() -> Self {
        CtrlFrame {
            opcode:      0x10,
            start_types: Box::new([]),
            end_types:   Box::new([]),
            height:      0,
            unreachable: false,
        }
    }
}

pub struct Validation<'a> {
    #[allow(dead_code)]
    context:    &'a ValidationContext,
    val_stack:  Vec<ValueType>,
    ctrl_stack: Vec<CtrlFrame>,
}

impl<'a> Validation<'a> {
    pub fn new(context: &'a ValidationContext) -> Validation<'a> {
        let ctrl_stack = vec![CtrlFrame::default()];
        Validation {
            context,
            val_stack: Vec::default(),
            ctrl_stack,
        }
    }

    fn push_val(&mut self, v: ValueType) {
        self.val_stack.push(v);
    }

    fn push_vals(&mut self, vals: &[ValueType]) {
        for v in vals.iter().rev() {
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

    #[allow(dead_code)]
    fn pop_vals(&mut self, expects: &[ValueType]) -> Result<Vec<ValidationType>> {
        let mut result: Vec<ValidationType> = vec![];
        for e in expects.iter().rev() {
            result.push(self.pop_expect(*e)?);
        }
        Ok(result)
    }

    #[allow(dead_code)]
    fn push_ctrl(&mut self, opcode: u8, start_types: &ParamsType, end_types: &ResultType) {
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

    #[allow(dead_code)]
    fn pop_ctrl(&mut self) -> Result<CtrlFrame> {
        let frame = self
            .ctrl_stack
            .pop()
            .ok_or(ValidationError::CtrlStackUnderflow)?;
        let vals = self.pop_vals(&frame.end_types)?;
        (vals.len() == frame.height).true_or(ValidationError::UnusedValues)?;
        Ok(frame)
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
