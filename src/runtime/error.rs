use {
    super::instance::ExternalVal,
    crate::syntax::{ImportDesc, Resolved},
    std::fmt,
};

#[macro_export]
macro_rules! impl_bug {
    ( $fmt:literal $(, $( $arg:expr ),*)? ) => {
        $crate::runtime::error::RuntimeErrorKind::ValidationError(
            format!($fmt$(, $($arg,)*)?)
        ).error()
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
    ImportNotFound(String, String),
    ImportMismatch(ImportDesc<Resolved>, ExternalVal),
    ValidationError(String),
    ArgumentCountError { expected: usize, got: usize },
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
        RuntimeErrorKind::Trap(tk).error()
    }
}

impl RuntimeErrorKind {
    pub fn error(self) -> RuntimeError {
        RuntimeError {
            kind:    self,
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

pub trait WithContext<T> {
    fn ctx<S: Into<String>>(self, msg: impl Fn() -> S) -> T;
}

impl<T> WithContext<Result<T>> for Result<T> {
    fn ctx<S: Into<String>>(self, msg: impl Fn() -> S) -> Result<T> {
        self.map_err(|e| e.with_context(msg()))
    }
}
