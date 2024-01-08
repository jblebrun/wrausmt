use {
    super::instance::ExternalVal,
    crate::{
        syntax::{ImportDesc, Resolved},
        validation::ValidationError,
    },
    std::fmt,
};

/// An impl_bug is a place where we're doing a runtime check for something
/// that should have been handled by module validation.
#[macro_export]
macro_rules! impl_bug {
    ( $fmt:literal $(, $( $arg:expr ),*)? ) => {
        $crate::runtime::error::RuntimeErrorKind::ImplementationBug(
            format!($fmt$(, $($arg,)*)?)
        )
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    context:  Vec<String>,
}

impl RuntimeError {
    pub fn with_context(mut self, msg: impl Into<String>) -> Self {
        self.context.push(msg.into());
        self
    }

    pub fn as_trap_error(&self) -> Option<&TrapKind> {
        match self.kind {
            RuntimeErrorKind::Trap(ref tk) => Some(tk),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum RuntimeErrorKind {
    MethodNotFound(String),
    ModuleNotFound(String),
    TypeNotFound(u32),
    ImportNotFound(String, String),
    ImportMismatch(ImportDesc<Resolved>, ExternalVal),
    ImplementationBug(String),
    ValidationError(ValidationError),
    ArgumentCountError { expected: usize, got: usize },
    CallStackExhaustion,
    Trap(TrapKind),
}

#[derive(Debug)]
pub enum TrapKind {
    IntegerDivideByZero,
    IntegerOverflow,
    UninitializedElement,
    OutOfBoundsMemoryAccess(usize, usize),
    OutOfBoundsTableAccess(usize, usize),
    Unreachable,
    UndefinedElement,
    CallIndirectTypeMismatch,
    InvalidConversionToInteger,
}

impl From<TrapKind> for RuntimeError {
    fn from(tk: TrapKind) -> RuntimeError {
        RuntimeErrorKind::Trap(tk).into()
    }
}

impl From<RuntimeErrorKind> for RuntimeError {
    fn from(value: RuntimeErrorKind) -> Self {
        RuntimeError {
            kind:    value,
            context: vec![],
        }
    }
}
impl From<ValidationError> for RuntimeError {
    fn from(value: ValidationError) -> Self {
        RuntimeError {
            kind:    RuntimeErrorKind::ValidationError(value),
            context: vec![],
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RuntimeError {}

pub type Result<T> = std::result::Result<T, RuntimeError>;
