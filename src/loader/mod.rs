/// The loader module is the bridge between the format parsing code, and the
/// runtime, which expects a fully resolved module as input.
use crate::format::{
    binary::error::BinaryParseError, binary::parse_wasm_data, text::parse::error::ParseError,
    text::parse_wast_data,
};
use {
    crate::{
        format::text::{lex::error::LexError, parse::error::ParseErrorKind},
        runtime::{error::RuntimeError, instance::ModuleInstance, Runtime},
    },
    std::{fs::File, io::Read, rc::Rc},
};

#[derive(Debug)]
pub enum LoaderError {
    IoError(std::io::Error),
    ParseError(ParseError),
    BinaryParseError(BinaryParseError),
    GenericError(Box<dyn std::error::Error>),
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

impl From<LexError> for LoaderError {
    fn from(e: LexError) -> Self {
        LoaderError::ParseError(ParseError::new_nocontext(ParseErrorKind::LexError(e)))
    }
}

impl From<RuntimeError> for LoaderError {
    fn from(e: RuntimeError) -> Self {
        LoaderError::GenericError(Box::new(e))
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
        self.load_wast_data(&mut File::open(filename)?)
    }

    fn load_wasm(&mut self, filename: &str) -> Result<Rc<ModuleInstance>> {
        self.load_wasm_data(&mut File::open(filename)?)
    }

    fn load_wasm_data(&mut self, read: &mut impl Read) -> Result<Rc<ModuleInstance>>;

    fn load_wast_data(&mut self, read: &mut impl Read) -> Result<Rc<ModuleInstance>>;
}

impl Loader for Runtime {
    fn load_wasm_data(&mut self, reader: &mut impl Read) -> Result<Rc<ModuleInstance>> {
        let module = parse_wasm_data(reader)?;
        let mod_inst = self.load(module)?;
        Ok(mod_inst)
    }

    fn load_wast_data(&mut self, reader: &mut impl Read) -> Result<Rc<ModuleInstance>> {
        let module = parse_wast_data(reader)?;
        let mod_inst = self.load(module)?;
        Ok(mod_inst)
    }
}
