/// The loader module is the bridge between the format parsing code, and the
/// runtime, which expects a fully resolved module as input.
use crate::{
    binary::{error::BinaryParseError, parse_wasm_data},
    compiler::{compile_module, ValidationError},
    text::parse::error::ParseError,
    text::{parse_wast_data, resolve::ResolveError},
};
use {
    std::{io::Read, rc::Rc},
    wrausmt_runtime::runtime::{error::RuntimeError, instance::ModuleInstance, Runtime},
};

#[derive(Debug)]
pub enum LoaderError {
    IoError(std::io::Error),
    ParseError(ParseError),
    BinaryParseError(BinaryParseError),
    RuntimeError(RuntimeError),
    ValidationError(ValidationError),
    ResolveError(ResolveError),
}

impl LoaderError {
    pub fn is_parse_error(&self) -> bool {
        matches!(self, Self::ParseError(_) | Self::BinaryParseError(_))
    }
}

impl std::error::Error for LoaderError {}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for LoaderError {
    fn from(e: std::io::Error) -> Self {
        LoaderError::IoError(e)
    }
}

impl From<ParseError> for LoaderError {
    fn from(e: ParseError) -> Self {
        LoaderError::ParseError(e)
    }
}

impl From<RuntimeError> for LoaderError {
    fn from(e: RuntimeError) -> Self {
        LoaderError::RuntimeError(e)
    }
}

impl From<BinaryParseError> for LoaderError {
    fn from(e: BinaryParseError) -> Self {
        LoaderError::BinaryParseError(e)
    }
}

impl From<ValidationError> for LoaderError {
    fn from(e: ValidationError) -> Self {
        LoaderError::ValidationError(e)
    }
}

impl From<ResolveError> for LoaderError {
    fn from(e: ResolveError) -> Self {
        LoaderError::ResolveError(e)
    }
}

pub type Result<T> = std::result::Result<T, LoaderError>;

pub trait Loader {
    fn load_wasm_data(&mut self, read: &mut impl Read) -> Result<Rc<ModuleInstance>>;

    fn load_wast_data(&mut self, read: &mut impl Read) -> Result<Rc<ModuleInstance>>;
}

impl Loader for Runtime {
    fn load_wasm_data(&mut self, reader: &mut impl Read) -> Result<Rc<ModuleInstance>> {
        let module = parse_wasm_data(reader)?;
        // TODO Switch to fail when validation is complete.
        let compiled = compile_module(module)?;
        let mod_inst = self.load(compiled)?;
        Ok(mod_inst)
    }

    fn load_wast_data(&mut self, reader: &mut impl Read) -> Result<Rc<ModuleInstance>> {
        let module = parse_wast_data(reader)?;
        // TODO Switch to fail when validation is complete.
        let compiled = compile_module(module)?;
        let mod_inst = self.load(compiled)?;
        Ok(mod_inst)
    }
}
