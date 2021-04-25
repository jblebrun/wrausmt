pub mod lex;
pub mod token;

pub mod module;

use crate::{err, types::{NumType, RefType, ValueType}};
use crate::error::Result;
use lex::Tokenizer;
use std::io::Read;
use token::{FileToken, Token};

pub struct Parser<R: Read> {
    tokenizer: Tokenizer<R>,
    current: FileToken,
    // 1 token of lookahead
    next: FileToken,
}

// Implementation for the basic token-handling methods.
impl<R: Read> Parser<R> {
    pub fn new(tokenizer: Tokenizer<R>) -> Result<Parser<R>> {
        let mut p = Parser {
            tokenizer,
            current: FileToken::default(),
            next: FileToken::default(),
        };
        p.advance()?;
        p.advance()?;
        Ok(p)
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
        println!("TOKEN IS NOW {:?}", self.current.token);
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

    fn expect_string(&mut self) -> Result<String> {
        match self.current.token {
            Token::String(ref mut data) => {
                let data = std::mem::take(data);
                self.advance()?;
                Ok(data)
            },
            _ => err!("expected string, not {:?}", self.current)
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

    fn try_unsigned(&mut self) -> Result<Option<u64>> {
        match self.current.token {
            Token::Unsigned(ref mut val) => {
                let val = std::mem::take(val);
                self.advance()?;
                Ok(Some(val))
            }
            _ => Ok(None)
        }
    }

    fn expect_valtype(&mut self) -> Result<ValueType> {
        match self.try_valtype() {
            Ok(Some(vt)) => Ok(vt),
            Ok(None) => err!("expected value type"),
            Err(e) => Err(e)
        }
    }

    fn try_valtype(&mut self) -> Result<Option<ValueType>> {
        let result = match &self.current.token {
            Token::Keyword(kw) => match kw.as_str() {
                "func" | "funcref" => Some(ValueType::Ref(RefType::Func)),
                "extern" | "externref" => Some(ValueType::Ref(RefType::Extern)),
                "i32" => Some(ValueType::Num(NumType::I32)),
                "i64" => Some(ValueType::Num(NumType::I64)),
                "f32" => Some(ValueType::Num(NumType::F32)),
                "f64" => Some(ValueType::Num(NumType::F64)),
                _ => None
            }
            _ => None
        };
        if result.is_some() {
            self.advance()?;
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
}
