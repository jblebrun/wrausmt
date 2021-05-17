use std::fmt;

use crate::syntax::{ImportDesc, Resolved};

use super::instance::ExternalVal;

#[macro_export]
macro_rules! impl_bug {
    ( $fmt:literal $(, $( $arg:expr ),*)? ) => {
        crate::runtime::error::RuntimeErrorKind::ValidationError(
            format!($fmt$(, $($arg,)*)?)
        ).error()
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    kind: RuntimeErrorKind,
    context: Vec<String>,
}

impl RuntimeError {
    pub fn with_context<S: Into<String>>(mut self, msg: S) -> Self {
        self.context.push(msg.into());
        self
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
    Trap(String),
}

impl RuntimeErrorKind {
    pub fn error(self) -> RuntimeError {
        RuntimeError {
            kind: self,
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
    fn ctx<F, S>(self, msg: F) -> T
    where
        F: Fn() -> S,
        S: Into<String>;
}

impl<T> WithContext<Result<T>> for Result<T> {
    fn ctx<F, S>(self, msg: F) -> Result<T>
    where
        F: Fn() -> S,
        S: Into<String>,
    {
        self.map_err(|e| e.with_context(msg()))
    }
}
