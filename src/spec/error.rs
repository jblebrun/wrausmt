use crate::{
    format::Location,
    loader::LoaderError,
    runtime::{error::RuntimeError, values::Value},
};

use super::format::{Action, ActionResult};

#[derive(Default)]
pub struct Failures {
    pub failures: Vec<Failure>,
}

impl std::fmt::Debug for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Test {} Line {}", self.testindex, self.location.line)?;
        writeln!(f, "{:?}\n", self.err)
    }
}

impl std::fmt::Display for Failures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl std::fmt::Debug for Failures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for failure in &self.failures {
            writeln!(f, "{:?}", failure)?;
        }
        writeln!(f, "{} failures", self.failures.len())
    }
}

pub struct Failure {
    location: Location,
    testindex: u32,
    err: SpecTestError,
}

#[derive(Debug)]
pub enum SpecTestError {
    NoModule(Action),
    Failures(Failures),
    LoaderError(LoaderError),
    InvocationError(RuntimeError),
    ResultLengthMismatch {
        results: Vec<Value>,
        expects: Vec<ActionResult>,
    },
    ResultMismatch {
        result: Value,
        expect: ActionResult,
    },
}

impl SpecTestError {
    pub fn into_failure(self, location: Location, testindex: u32) -> Failure {
        Failure {
            location,
            testindex,
            err: self,
        }
    }
}

impl std::fmt::Display for SpecTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl std::error::Error for SpecTestError {}

impl std::error::Error for Failures {}

pub type Result<T> = std::result::Result<T, SpecTestError>;

impl From<RuntimeError> for SpecTestError {
    fn from(e: RuntimeError) -> Self {
        SpecTestError::InvocationError(e)
    }
}

impl From<LoaderError> for SpecTestError {
    fn from(e: LoaderError) -> Self {
        SpecTestError::LoaderError(e)
    }
}
