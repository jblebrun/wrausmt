/// The loader module is the bridge between the format parsing code, and the
/// runtime, which expects a fully resolved module as input.
use crate::{
    binary::{error::BinaryParseError, parse_wasm_data},
    text::parse::error::ParseError,
    text::parse_wast_data,
};
use {
    std::{fs::File, io::Read, rc::Rc},
    wrausmt_runtime::{
        runtime::{error::RuntimeError, instance::ModuleInstance, Runtime},
        validation::ValidationMode,
    },
};

#[derive(Debug)]
pub enum LoaderError {
    IoError(std::io::Error),
    ParseError(ParseError),
    BinaryParseError(BinaryParseError),
    RuntimeError(RuntimeError),
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

pub type Result<T> = std::result::Result<T, LoaderError>;

pub trait Loader {
    fn load_wast(&mut self, filename: &str) -> Result<Rc<ModuleInstance>> {
        self.load_wast_data(&mut File::open(filename)?, ValidationMode::Warn)
    }

    fn load_wasm(&mut self, filename: &str) -> Result<Rc<ModuleInstance>> {
        self.load_wasm_data(&mut File::open(filename)?, ValidationMode::Warn)
    }

    fn load_wasm_data(
        &mut self,
        read: &mut impl Read,
        validation_mode: ValidationMode,
    ) -> Result<Rc<ModuleInstance>>;

    fn load_wast_data(
        &mut self,
        read: &mut impl Read,
        validation_mode: ValidationMode,
    ) -> Result<Rc<ModuleInstance>>;
}

impl Loader for Runtime {
    fn load_wasm_data(
        &mut self,
        reader: &mut impl Read,
        validation_mode: ValidationMode,
    ) -> Result<Rc<ModuleInstance>> {
        let module = parse_wasm_data(reader)?;
        // TODO Switch to fail when validation is complete.
        let mod_inst = self.load(module, validation_mode)?;
        Ok(mod_inst)
    }

    fn load_wast_data(
        &mut self,
        reader: &mut impl Read,
        validation_mode: ValidationMode,
    ) -> Result<Rc<ModuleInstance>> {
        let module = parse_wast_data(reader)?;
        // TODO Switch to fail when validation is complete.
        let mod_inst = self.load(module, validation_mode)?;
        Ok(mod_inst)
    }
}
