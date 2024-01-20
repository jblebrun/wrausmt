//! Validation for instructions sequences as defined in [Spec].
//!
//! [Spec]: https://webassembly.github.io/spec/core/appendix/algorithm.html

use {
    self::stacks::Stacks,
    wrausmt_runtime::{
        instructions::opcodes,
        syntax::{
            self,
            types::{GlobalType, MemType, RefType, TableType, ValueType},
            ImportDesc, Index, Instruction, LocalIndex, Module, Resolved, UncompiledExpr,
        },
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
    ExpectedRef {
        actual: ValidationType,
    },
    UnusedValues,
    UnknownLocal(Index<Resolved, LocalIndex>),
    AlignmentTooLarge(u32),
    UnhandledInstruction(Instruction<Resolved>),
    OpcodeMismatch,
    OperandsMismatch,
    LabelOutOfRange,
    BreakTypeMismatch,
    UnknownMemory,
    UnknownData,
    UnknownTable,
    UnknownElem,
    UnknownFunc,
    UnknownType,
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

impl From<syntax::FunctionType> for FunctionType {
    fn from(value: syntax::FunctionType) -> Self {
        FunctionType {
            params:  value.params.iter().map(|p| p.valuetype).collect(),
            results: value.results.iter().map(|r| r.valuetype).collect(),
        }
    }
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
    pub types:   Vec<FunctionType>,
    pub funcs:   Vec<FunctionType>,
    pub tables:  Vec<TableType>,
    pub mems:    Vec<MemType>,
    pub globals: Vec<GlobalType>,
    pub elems:   Vec<RefType>,
    pub datas:   usize,
}

impl ModuleContext {
    /// Create a new [`ModuleContext`] for validation, using the type
    /// information in the provided [`Module`]. The informatin will be copied
    /// out of the module.
    pub fn new(module: &Module<Resolved, UncompiledExpr<Resolved>>) -> Self {
        let mut funcs: Vec<FunctionType> = Vec::new();
        let mut tables: Vec<TableType> = Vec::new();
        let mut mems: Vec<MemType> = Vec::new();
        let mut globals: Vec<GlobalType> = Vec::new();

        for import in module.imports.iter() {
            match &import.desc {
                ImportDesc::Func(tu) => funcs.push(
                    module.types[tu.index().value() as usize]
                        .functiontype
                        .clone()
                        .into(),
                ),
                ImportDesc::Table(tt) => tables.push(tt.clone()),
                ImportDesc::Mem(mt) => mems.push(mt.clone()),
                ImportDesc::Global(gt) => globals.push(gt.clone()),
            }
        }
        funcs.extend(module.funcs.iter().map(|f| {
            module.types[f.typeuse.index().value() as usize]
                .functiontype
                .clone()
                .into()
        }));

        tables.extend(module.tables.iter().map(|t| t.tabletype.clone()));
        mems.extend(module.memories.iter().map(|m| m.memtype.clone()));
        globals.extend(module.globals.iter().map(|g| g.globaltype.clone()));

        ModuleContext {
            types: module
                .types
                .iter()
                .map(|t| FunctionType {
                    params:  t.functiontype.params.iter().map(|p| p.valuetype).collect(),
                    results: t.functiontype.results.iter().map(|r| r.valuetype).collect(),
                })
                .collect(),
            funcs,
            tables,
            mems,
            globals,
            elems: module.elems.iter().map(|e| e.elemlist.reftype).collect(),
            datas: module.data.len(),
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
