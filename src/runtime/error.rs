use std::fmt;

use crate::syntax::{ImportDesc, Resolved};

use super::instance::ExternalVal;

#[macro_export]
macro_rules! impl_bug {
    ( $fmt:literal $(, $( $arg:expr ),*)? ) => {
        crate::runtime::error::RuntimeError::ValidationError(
            format!($fmt$(, $($arg,)*)?)
        )
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    MethodNotFound(String),
    ModuleNotFound(String),
    ImportNotFound(String, String),
    ImportMismatch(ImportDesc<Resolved>, ExternalVal),
    ValidationError(String),
    ArgumentCountError { expected: usize, got: usize },
    Trap(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RuntimeError {}

pub type Result<T> = std::result::Result<T, RuntimeError>;
