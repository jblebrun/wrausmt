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
use crate::error::Result;
use lex::Tokenizer;
use std::io::Read;
use token::{FileToken, Token};

use self::{data::DataField, elem::ElemField, export::ExportField, func::FuncField, global::GlobalField, import::ImportField, memory::MemoryField, start::StartField, table::TableField, typefield::TypeField};

pub struct Parser<R: Read> {
    tokenizer: Tokenizer<R>,
    current: FileToken,
    // 1 token of lookahead
    next: FileToken,
}

// Recursive descent parsing. So far, the grammar for the text format seems to be LL(1),
// so recursive descent works out really nicely.
// TODO - return an iterator of fields instead?
impl<R: Read> Parser<R> {
    pub fn parse(r: R) -> Result<Vec<Field>> {
        let mut parser = Parser::new(r)?;
        parser.advance()?;
        parser.advance()?;
        parser.parse_module()
    }

    fn new(r: R) -> Result<Parser<R>> {
        Ok(Parser {
            tokenizer: Tokenizer::new(r)?,
            current: FileToken::default(),
            next: FileToken::default(),
        })
    }

    // Updates the lookahead token to the next value
    // provided by the tokenizer.
    fn next(&mut self) -> Result<()> {
        if self.next.token == Token::Eof {
            return err!("Attempted to advance past EOF")
        }
        
        match self.tokenizer.next() {
            None => self.next.token = Token::Eof,
            Some(Ok(t)) => self.next = t,
            Some(Err(e)) => return Err(e),
        }
        Ok(())
    }

    // Advance to the next token, skipping all whitespace and comments.
    // Returns the current token to be owned by caller.
    fn advance(&mut self) -> Result<Token> {
        let out: Token = std::mem::take(&mut self.current.token);
        self.current = std::mem::take(&mut self.next);
        self.next()?;
        while self.next.token.ignorable() {
            self.next()?;
        }
        Ok(out)
    }

    fn at_expr_start(&mut self, name: &str) -> Result<bool> {
        if self.current.token != Token::Open {
            return Ok(false) 
        }
        match &self.next.token {
            Token::Keyword(k) if k == name => {
                self.advance()?;
                self.advance()?;
                Ok(true) 
            }
            _ => Ok(false)
        }
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

        // section*
        let mut result: Vec<Field> = vec![];
        while let Some(s) = self.parse_section()? {
            result.push(s);

            match self.current.token {
                Token::Open => (),
                Token::Close => break,
                _ => return err!("Invalid start token {:?}", self.current),
            }
        }

        Ok(result)
    }

    fn consume_expression(&mut self) -> Result<()> {
     let mut depth = 1;
        while depth > 0 {
            match self.current.token {
                Token::Open => depth += 1,
                Token::Close => depth -= 1,
                _ => (),
            };
            if depth == 0 {
                break;
            }
            self.advance()?;
        }
        self.advance()?;
        Ok(())
    }

    // Parser should be located at the token immediately following a '('
    fn parse_section(&mut self) -> Result<Option<Field>> {
        if let Some(f) = self.parse_type_field()? { return Ok(Some(f)) }
        if let Some(f) = self.parse_func_field()? { return Ok(Some(f)) }
        if let Some(f) = self.parse_table_field()? { return Ok(Some(f)) }
        if let Some(f) = self.parse_memory_field()? { return Ok(Some(f)) }
        if let Some(f) = self.parse_import_field()? { return Ok(Some(f)) }
        if let Some(f) = self.parse_export_field()? { return Ok(Some(f)) }
        if let Some(f) = self.parse_global_field()? { return Ok(Some(f)) }
        if let Some(f) = self.parse_start_field()? { return Ok(Some(f)) }
        if let Some(f) = self.parse_elem_field()? { return Ok(Some(f)) }
        if let Some(f) = self.parse_data_field()? { return Ok(Some(f)) }
        return err!("no section found at {:?} {:?}", self.current, self.next)
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
