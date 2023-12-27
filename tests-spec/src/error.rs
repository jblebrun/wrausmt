use {
    super::format::ActionResult,
    wrausmt::{
        format::Location,
        loader::LoaderError,
        runtime::{
            error::{RuntimeError, TrapKind},
            values::Value,
        },
        syntax::Id,
    },
};

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
    location:  Location,
    testindex: usize,
    err:       SpecTestError,
}

#[derive(Debug)]
pub enum SpecTestError {
    NoModule(Option<Id>),
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
    TrapMismatch {
        result: Option<Box<SpecTestError>>,
        expect: String,
    },
    RegisterMissingModule(String),
    UnImplemented,
}

impl SpecTestError {
    pub fn into_failure(self, location: Location, testindex: usize) -> Failure {
        Failure {
            location,
            testindex,
            err: self,
        }
    }

    pub fn is_parse_error(&self) -> bool {
        matches!(self, Self::LoaderError(le) if le.is_parse_error())
    }

    pub fn as_trap_error(&self) -> Option<&TrapKind> {
        match self {
            Self::InvocationError(re) => re.as_trap_error(),
            _ => None,
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

impl From<std::io::Error> for SpecTestError {
    fn from(e: std::io::Error) -> Self {
        SpecTestError::LoaderError(e.into())
    }
}
