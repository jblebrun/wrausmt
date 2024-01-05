use {
    crate::format::ActionResult,
    wrausmt_format::{loader::LoaderError, text::location::Location},
    wrausmt_runtime::{
        runtime::{error::RuntimeError, values::Value},
        syntax::Id,
    },
};

pub type Result<T> = std::result::Result<T, SpecTestError>;

/// SpecTestError is the result of attempting to load and run one single spec
/// script. It will either be a LoadError (due to parsing issues), or a Failures
/// error, containing one or more test failures.
#[derive(Debug)]
pub enum SpecTestError {
    LoaderError(LoaderError),
    Failures(Vec<Failure>),
}

/// A Failure is a single test failure, including the file location and test
/// index of the failed command.
pub struct Failure {
    pub location:   Location,
    pub test_index: usize,
    pub err:        CmdError,
}

/// CmdError is the set of results that can occur when trying to execute one
/// command in a spec script.
#[derive(Debug)]
pub enum CmdError {
    LoaderError(LoaderError),
    RegisterMissingModule(String),
    NoModule(Option<Id>),
    InvocationError(RuntimeError),
    TestFailure(TestFailureError),
    UnImplemented,
}

/// A TestFailure is returned when the command successfully completes, but the
/// returned results do not match the expectations.
#[derive(Debug)]
pub enum TestFailureError {
    ResultLengthMismatch {
        results: Vec<Value>,
        expects: Vec<ActionResult>,
    },
    ResultMismatch {
        result: Value,
        expect: ActionResult,
    },
    FailureMismatch {
        result: Option<Box<CmdError>>,
        expect: String,
    },
}

impl TestFailureError {
    pub fn failure_mismatch(failure: &str, error: CmdError) -> TestFailureError {
        TestFailureError::FailureMismatch {
            result: Some(Box::new(error)),
            expect: failure.to_string(),
        }
    }

    pub fn failure_missing(failure: &str) -> TestFailureError {
        TestFailureError::FailureMismatch {
            result: None,
            expect: failure.to_string(),
        }
    }
}

impl std::error::Error for SpecTestError {}
impl std::fmt::Display for SpecTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}
impl std::fmt::Debug for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Test {} Line {}", self.test_index, self.location.line)?;
        writeln!(f, "{:?}\n", self.err)
    }
}
impl std::error::Error for CmdError {}
impl std::fmt::Display for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl std::error::Error for TestFailureError {}
impl std::fmt::Display for TestFailureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl From<LoaderError> for SpecTestError {
    fn from(e: LoaderError) -> Self {
        SpecTestError::LoaderError(e)
    }
}
impl From<std::io::Error> for SpecTestError {
    fn from(e: std::io::Error) -> Self {
        SpecTestError::LoaderError(e.into())
    }
}
impl From<RuntimeError> for CmdError {
    fn from(e: RuntimeError) -> Self {
        CmdError::InvocationError(e)
    }
}
impl From<LoaderError> for CmdError {
    fn from(e: LoaderError) -> Self {
        CmdError::LoaderError(e)
    }
}
impl From<TestFailureError> for CmdError {
    fn from(e: TestFailureError) -> Self {
        CmdError::TestFailure(e)
    }
}
