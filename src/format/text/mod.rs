mod lex;
mod token;

pub mod typefield;
pub mod func;
pub mod table;
pub mod memory;
pub mod import;
pub mod export;
pub mod global;
pub mod start;
pub mod elem;
pub mod data;

use crate::{err, types::{NumType, RefType, ValueType}};
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

    fn expect_close(&mut self) -> Result<()> {
        match self.current.token {
            Token::Close => {
                self.advance()?;
                Ok(())
            },
            _ => err!("expected close, not {:?}", self.current)
        }
    }

    fn try_id(&mut self) -> Result<Option<String>> {
        match self.current.token {
            Token::Id(ref mut id) => {
                let id = std::mem::take(id);
                self.advance()?;
                Ok(Some(id))
            },
            _ => Ok(None)
        }
    }

    fn expect_valtype(&mut self) -> Result<ValueType> {
        let result = match &self.current.token {
            Token::Keyword(kw) => match kw.as_str() {
                "func" | "funcref" => ValueType::Ref(RefType::Func),
                "extern" | "externref" => ValueType::Ref(RefType::Extern),
                "i32" => ValueType::Num(NumType::I32),
                "i64" => ValueType::Num(NumType::I64),
                "f32" => ValueType::Num(NumType::F32),
                "f64" => ValueType::Num(NumType::F64),
                _ => return err!("{} is not a value type", kw) 
            }
            _ => return err!("{:?} is not a value type", self.current.token)
        };
        self.advance()?;
        Ok(result)
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum Index {
   Numeric(u32),
   Named(String)
}

#[derive(Debug, Default, PartialEq)]
pub struct Expr {
}

// param := (param id? valtype)
#[derive(Debug, PartialEq)]
pub struct FParam {
    pub id: Option<String>,
    pub valuetype: ValueType,
}

#[macro_export]
macro_rules! fparam {
    ( $pid:literal; $vt:ident ) => {
        wrausmt::fparam! { Some($pid.to_string()); $vt }
    };
    ( $id:expr; Func ) => {
        FParam{ id: $id, valuetype: wrausmt::types::RefType::Func.into() }
    };
    ( $id:expr; Extern ) => {
        FParam{ id: $id, valuetype: wrausmt::types::RefType::Extern.into() }
    };
    ( $id:expr; $vt:ident ) => {
        FParam{ id: $id, valuetype: wrausmt::types::NumType::$vt.into() }
    };
    ( $vt:ident ) => {
        wrausmt::fparam! { None; $vt }
    }
}

#[macro_export]
macro_rules! fresult {
    ( $vt:ident ) => {
        FResult{ valuetype: wrausmt::types::NumType::$vt.into() }
    }
}

// result := (result valtype)
#[derive(Debug, PartialEq)]
pub struct FResult {
    pub valuetype: ValueType,
}

impl <R: Read> Parser<R> {
    fn try_parse_fresult(&mut self) -> Result<Option<FResult>> {
        if !self.at_expr_start("result")? {
            return Ok(None);
        }
        
        let valuetype = self.expect_valtype()?;
        
        self.expect_close()?;

        Ok(Some(FResult { valuetype }))
    }
    fn try_parse_fparam(&mut self) -> Result<Option<FParam>> {
        if !self.at_expr_start("param")? {
            return Ok(None);
        }

        let id = self.try_id()?;
        
        let valuetype = self.expect_valtype()?;
        
        self.expect_close()?;

        Ok(Some(FParam { id, valuetype }))
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct TypeUse {
    params: Vec<FParam>,
    result: Vec<FResult>
}
