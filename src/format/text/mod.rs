mod lex;
mod token;

mod typefield;
mod func;
mod table;
mod memory;
mod import;
mod export;
mod global;
mod start;
mod elem;
mod data;

use crate::{err, types::ValueType};
use crate::error::{Result, ResultFrom};
use lex::Tokenizer;
use std::io::Read;
use token::{FileToken, Token};

use self::{data::DataField, elem::ElemField, export::ExportField, func::FuncField, global::GlobalField, import::ImportField, memory::MemoryField, start::StartField, table::TableField, typefield::TypeField};

pub struct Parser<R: Read> {
    tokenizer: Tokenizer<R>,
    current: FileToken,
}

// Recursive descent parsing. So far, the grammar for the text format seems to be LL(1),
// so recursive descent works out really nicely.
// TODO - return an iterator of fields instead?
impl<R: Read> Parser<R> {
    pub fn parse(r: R) -> Result<Vec<Field>> {
        let mut parser = Parser::new(r)?;
        parser.advance()?;
        parser.parse_module()
    }

    fn new(r: R) -> Result<Parser<R>> {
        Ok(Parser {
            tokenizer: Tokenizer::new(r)?,
            current: FileToken::default(),
        })
    }

    fn next(&mut self) -> Result<()> {
        if self.current.token == Token::Eof {
            panic!("EOF")
        }
        match self.tokenizer.next() {
            None => self.current.token = Token::Eof,
            Some(Ok(t)) => self.current = t,
            Some(Err(e)) => return Err(e),
        }
        Ok(())
    }

    // Advance to the next token, skipping all whitespace and comments.
    fn advance(&mut self) -> Result<()> {
        self.next()?;
        while self.current.token.ignorable() {
            self.next()?;
        }
        Ok(())
    }


    /// Attempt to parse the current token stream as a WebAssembly module.
    /// On success, a vector of sections is returned. They can be organized into a
    /// module object.
    fn parse_module(&mut self) -> Result<Vec<Field>> {
        if self.current.token != Token::Open {
            return err!("Invalid start token {:?}", self.current);
        }
        self.advance()?;

        // Modules usually start with "(module". However, this is optional, and a module file can
        // be a list of top-levelo sections.
        if self.current.token.is_keyword("module") {
            self.advance()?;
        }

        if self.current.token != Token::Open {
            return err!("Invalid start token {:?}", self.current);
        }
        self.advance()?;

        // section*
        let mut result: Vec<Field> = vec![];
        while let Some(s) = self.parse_section()? {
            result.push(s);

            self.advance()?;
            match self.current.token {
                Token::Open => (),
                Token::Close => break,
                _ => return err!("Invalid start token {:?}", self.current),
            }
            self.advance()?;
        }

        Ok(result)
    }

    fn consume_expression(&mut self) -> Result<()> {
     let mut depth = 1;
        while depth > 0 {
            self.advance()?;
            match self.current.token {
                Token::Open => depth += 1,
                Token::Close => depth -= 1,
                _ => (),
            };
            if depth == 0 {
                break;
            }
        }
        Ok(())
    }

    // Parser should be located at the token immediately following a '('
    fn parse_section(&mut self) -> Result<Option<Field>> {
        let name = self.current.token.expect_keyword()
            .wrap("parsing section name")?
            .clone();

        match name.as_ref() {
            // TODO - is there a cleaner way to map the inner result
            // than the double map construct?
            "type" => self.parse_type_field().map(|f| f.map(|f| f.into())),
            "func" => self.parse_func_field().map(|f| f.map(|f| f.into())),
            "table" => self.parse_table_section().map(|f| f.map(|f| f.into())),
            "memory" => self.parse_memory_section().map(|f| f.map(|f| f.into())),
            "import" => self.parse_import_section().map(|f| f.map(|f| f.into())),
            "export" => self.parse_export_section().map(|f| f.map(|f| f.into())),
            "global" => self.parse_global_section().map(|f| f.map(|f| f.into())),
            "start" => self.parse_start_section().map(|f| f.map(|f| f.into())),
            "elem" => self.parse_elem_section().map(|f| f.map(|f| f.into())),
            "data" => self.parse_data_section().map(|f| f.map(|f| f.into())),
            _ => err!("unknown section name {}", name)
        }
    }
}

#[derive(Debug)]
pub enum Field {
    Type(TypeField),
    Func(FuncField),
    Table(TableField),
    Memory(MemoryField),
    Import(ImportField),
    Export(ExportField),
    Global(GlobalField),
    Start(StartField),
    Elem(ElemField),
    Data(DataField),
}

macro_rules! from_field {
    ( $n:ident, $nf:ident ) => {
        impl From<$nf> for Field {
            fn from(f: $nf) -> Self {
                Field::$n(f)
            }
        }
    }
}

from_field! { Type, TypeField }
from_field! { Func, FuncField }
from_field! { Table, TableField }
from_field! { Memory, MemoryField }
from_field! { Import, ImportField }
from_field! { Export, ExportField }
from_field! { Global, GlobalField }
from_field! { Start, StartField }
from_field! { Elem, ElemField }
from_field! { Data, DataField }

#[derive(Debug)]
#[allow(dead_code)]
enum Index {
   Numeric(u32),
   Named(String)
}

#[derive(Debug, Default)]
pub struct Expr {
}

// param := (param id? valtype)
#[derive(Debug)]
pub struct FParam {
    id: Option<String>,
    typ: ValueType,
}

// param := (result valtype)
#[derive(Debug)]
pub struct FResult {
    typ: ValueType,
}

#[derive(Debug, Default)]
pub struct TypeUse {
    params: Vec<FParam>,
    result: Vec<FResult>
}
