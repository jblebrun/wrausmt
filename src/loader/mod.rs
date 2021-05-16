use std::rc::Rc;

use crate::format::text::lex::Tokenizer;
use crate::format::text::parse::Parser;
use crate::format::{
    binary::error::BinaryParseError, binary::parse, text::parse::error::ParseError,
};
use crate::runtime::{instance::ModuleInstance, Runtime};
use crate::syntax::{Module, Resolved};
use crate::{format::text::lex::error::LexError, runtime::error::RuntimeError};

#[derive(Debug)]
pub enum LoaderError {
    IoError(std::io::Error),
    LexError(LexError),
    ParseError(ParseError),
    BinaryParseError(BinaryParseError),
    GenericError(Box<dyn std::error::Error>),
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

impl From<LexError> for LoaderError {
    fn from(e: LexError) -> Self {
        LoaderError::LexError(e)
    }
}

impl From<ParseError> for LoaderError {
    fn from(e: ParseError) -> Self {
        LoaderError::ParseError(e)
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
    fn load_wast(&mut self, filename: &str) -> Result<Rc<ModuleInstance>>;
    fn load_wasm(&mut self, filename: &str) -> Result<Rc<ModuleInstance>>;
    fn load_wasm_data(&mut self, filename: &[u8]) -> Result<Rc<ModuleInstance>>;
}

pub fn load_ast(filename: &str) -> Result<Module<Resolved>> {
    let f = std::fs::File::open(filename)?;

    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let ast = parser.parse_full_module()?;
    Ok(ast)
}

impl Loader for Runtime {
    fn load_wast(&mut self, filename: &str) -> Result<Rc<ModuleInstance>> {
        let ast = load_ast(filename)?;

        println!("MODULE {:#?}", ast);

        let mod_inst = self.load(ast)?;
        Ok(mod_inst)
    }

    fn load_wasm(&mut self, filename: &str) -> Result<Rc<ModuleInstance>> {
        let mut f = std::fs::File::open(filename)?;

        let module = parse(&mut f)?;

        let mod_inst = self.load(module)?;
        Ok(mod_inst)
    }

    fn load_wasm_data(&mut self, data: &[u8]) -> Result<Rc<ModuleInstance>> {
        let module = parse(data)?;

        let mod_inst = self.load(module)?;
        Ok(mod_inst)
    }
}
