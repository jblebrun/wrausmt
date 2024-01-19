//! Validation for instructions sequences as defined in [Spec].
//!
//! [Spec]: https://webassembly.github.io/spec/core/appendix/algorithm.html

use {
    self::stacks::Stacks,
    wrausmt_runtime::{
        instructions::opcodes,
        syntax::{types::ValueType, Index, LocalIndex, Module, Opcode, Resolved, UncompiledExpr},
    },
};

mod ops;
mod stacks;

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
    AlignmentTooLarge(u32),
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

#[derive(Clone, Debug, Default)]
pub struct FunctionType {
    pub params:  Vec<ValueType>,
    pub results: Vec<ValueType>,
}

/// A simple struct containing the type information needed for validation of the
/// module. It contains all of the items in the context for the current module.
///
/// The other types are represented as follows:
/// * locals: managed directly in [`Validation`].
/// * labels: managed implicitly in [`Validation`] via the `ctrl_stack`.
/// * return: managed implicitly when validating a function via the first frame
///   that's pushed to the `ctrl_strack`.
///
/// [Spec]: https://webassembly.github.io/spec/core/valid/conventions.html#context
#[derive(Clone, Debug, Default)]
pub struct ModuleContext {
    pub types: Vec<FunctionType>,
}

impl ModuleContext {
    /// Create a new [`ModuleContext`] for validation, using the type
    /// information in the provided [`Module`]. The informatin will be copied
    /// out of the module.
    pub fn new(module: &Module<Resolved, UncompiledExpr<Resolved>>) -> Self {
        ModuleContext {
            types: module
                .types
                .iter()
                .map(|t| FunctionType {
                    params:  t.functiontype.params.iter().map(|p| p.valuetype).collect(),
                    results: t.functiontype.results.iter().map(|r| r.valuetype).collect(),
                })
                .collect(),
        }
    }
}
/// The Validation context and implementation.
///
/// [Spec]: https://webassembly.github.io/spec/core/appendix/algorithm.html
pub struct Validation<'a> {
    pub mode: ValidationMode,

    // Module
    pub module: &'a ModuleContext,

    // Func
    pub localtypes: Vec<ValueType>,

    stacks: Stacks,
}

impl<'a> Validation<'a> {
    pub fn new(
        mode: ValidationMode,
        module: &ModuleContext,
        localtypes: Vec<ValueType>,
        resulttypes: Vec<ValueType>,
    ) -> Validation {
        let stacks = Stacks::new();
        let mut val = Validation {
            mode,
            module,
            localtypes,
            stacks,
        };

        val.stacks
            .push_ctrl(opcodes::BLOCK, Vec::new(), resulttypes);
        val
    }
}
