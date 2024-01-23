//! Validation for instructions sequences as defined in [Spec].
//!
//! [Spec]: https://webassembly.github.io/spec/core/appendix/algorithm.html

use {
    self::stacks::Stacks,
    super::ToValidationError,
    wrausmt_runtime::{
        instructions::opcodes,
        syntax::{
            self,
            location::Location,
            types::{GlobalType, MemType, RefType, TableType, ValueType},
            ImportDesc, Index, Instruction, LocalIndex, Module, Resolved, UncompiledExpr,
        },
    },
};

mod ops;
mod stacks;

#[derive(Debug)]
pub enum ValidationErrorKind {
    AlignmentTooLarge(u32),
    BreakTypeMismatch,
    CtrlStackUnderflow,

    ExpectedRef {
        actual: ValidationType,
    },
    ExpectedNum {
        actual: ValidationType,
    },
    ImmutableGlobal,
    InvalidConstantGlobal,
    InvalidConstantInstruction,
    MemoryTooLarge,
    OpcodeMismatch,
    OperandsMismatch,
    TableTooLarge,
    TypeMismatch {
        actual: ValidationType,
        expect: ValidationType,
    },
    UnhandledInstruction(Instruction<Resolved>),
    UnknownLocal(Index<Resolved, LocalIndex>),
    UnknownData,
    UnknownElem,
    UnknownFunc,
    UnknownGlobal,
    UnknownLabel,
    UnknownMemory,
    UnknownTable,
    UnknownType,
    UnusedValues,
    UnsupportedSelect,
    ValStackUnderflow,
    WrongTableType,
}

#[derive(Debug)]
pub struct ValidationError {
    kind:     ValidationErrorKind,
    location: Location,
}

impl ValidationError {
    pub fn new(kind: ValidationErrorKind, location: Location) -> ValidationError {
        ValidationError { kind, location }
    }

    pub fn kind(&self) -> &ValidationErrorKind {
        &self.kind
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?} {:?}", self.location, self.kind)
    }
}

impl std::error::Error for ValidationError {}

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
pub(crate) type KindResult<T> = std::result::Result<T, ValidationErrorKind>;

/// Type representations during validation.
///
/// [Spec]: https://webassembly.github.io/spec/core/appendix/algorithm.html#data-structures
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum ValidationType {
    #[default]
    Unknown,
    Value(ValueType),
}

impl<I: Into<ValueType>> From<I> for ValidationType {
    fn from(value: I) -> Self {
        ValidationType::Value(value.into())
    }
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

#[derive(Clone, Debug)]
pub struct GlobalValidationType {
    pub globaltype: GlobalType,
    pub imported:   bool,
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
#[derive(Clone, Debug)]

pub struct ModuleContext {
    pub types:   Vec<FunctionType>,
    pub funcs:   Vec<FunctionType>,
    pub tables:  Vec<TableType>,
    pub mems:    Vec<MemType>,
    pub globals: Vec<GlobalValidationType>,
    pub elems:   Vec<RefType>,
    pub datas:   usize,
}

impl ModuleContext {
    /// Create a new [`ModuleContext`] for validation, using the type
    /// information in the provided [`Module`]. The informatin will be copied
    /// out of the module.
    pub fn new(module: &Module<Resolved, UncompiledExpr<Resolved>>) -> Result<Self> {
        let mut funcs: Vec<FunctionType> = Vec::new();
        let mut tables: Vec<TableType> = Vec::new();
        let mut mems: Vec<MemType> = Vec::new();
        let mut globals: Vec<GlobalValidationType> = Vec::new();

        for import in module.imports.iter() {
            match &import.desc {
                ImportDesc::Func(tu) => funcs.push(
                    module
                        .types
                        .get(tu.index().value() as usize)
                        .ok_or(ValidationErrorKind::UnknownType)
                        .validation_error(import.location)?
                        .functiontype
                        .clone()
                        .into(),
                ),
                ImportDesc::Table(tt) => tables.push(tt.clone()),
                ImportDesc::Mem(mt) => mems.push(mt.clone()),
                ImportDesc::Global(gt) => globals.push(GlobalValidationType {
                    globaltype: gt.clone(),
                    imported:   true,
                }),
            }
        }

        for f in &module.funcs {
            funcs.push(
                module
                    .types
                    .get(f.typeuse.index().value() as usize)
                    .ok_or(ValidationErrorKind::UnknownType)
                    .validation_error(f.location)?
                    .functiontype
                    .clone()
                    .into(),
            );
        }

        tables.extend(module.tables.iter().map(|t| t.tabletype.clone()));
        mems.extend(module.memories.iter().map(|m| m.memtype.clone()));
        globals.extend(module.globals.iter().map(|g| GlobalValidationType {
            globaltype: g.globaltype.clone(),
            imported:   false,
        }));

        Ok(ModuleContext {
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
        })
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
