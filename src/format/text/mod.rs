pub mod lex;
pub mod token;

pub mod module;

use crate::{error::Result, types::Limits};
use crate::{
    err, error,
    types::{NumType, RefType, ValueType},
};
use lex::Tokenizer;
use std::io::Read;
use token::{FileToken, Token};

pub struct Parser<R: Read> {
    tokenizer: Tokenizer<R>,
    pub current: FileToken,
    // 1 token of lookahead
    pub next: FileToken,
}

type ParseFn<S, T> = fn(&mut S) -> Result<Option<T>>;
type ParseGroupFn<S, T> = fn(&mut S) -> Result<Option<Vec<T>>>;

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
            return err!("Attempted to advance past EOF");
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
    pub fn advance(&mut self) -> Result<Token> {
        let out: Token = std::mem::take(&mut self.current.token);
        self.current = std::mem::take(&mut self.next);
        self.next()?;
        while self.next.token.ignorable() {
            self.next()?;
        }
        //println!("TOKEN IS NOW {:?}", self.current.token);
        Ok(out)
    }

    pub fn try_expr_start(&mut self, name: &str) -> Result<bool> {
        if self.current.token != Token::Open {
            return Ok(false);
        }
        match &self.next.token {
            Token::Keyword(k) if k == name => {
                self.advance()?;
                self.advance()?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

   pub fn peek_expr_start(&mut self, name: &str) -> Result<bool> {
        if self.current.token != Token::Open {
            return Ok(false);
        }
        match &self.next.token {
            Token::Keyword(k) if k == name => {
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn expect_expr_start(&mut self, name: &str) -> Result<()> {
        if !self.try_expr_start(name)? {
            return err!("expected expression start ({}", name);
        }
        Ok(())
    }

    pub fn expect_close(&mut self) -> Result<()> {
        match self.current.token {
            Token::Close => {
                self.advance()?;
                Ok(())
            }
            _ => err!("expected close, not {:?}", self.current),
        }
    }

    pub fn try_string(&mut self) -> Result<Option<String>> {
        match self.current.token {
            Token::String(ref mut data) => {
                let data = std::mem::take(data);
                self.advance()?;
                Ok(Some(data))
            }
            _ => Ok(None)
        }
    }

    pub fn expect_string(&mut self) -> Result<String> {
        self.try_string()?.ok_or_else(|| error!("expected string"))
    }

    pub fn try_id(&mut self) -> Result<Option<String>> {
        match self.current.token {
            Token::Id(ref mut id) => {
                let id = std::mem::take(id);
                self.advance()?;
                Ok(Some(id))
            }
            _ => Ok(None),
        }
    }

    pub fn try_keyword(&mut self) -> Result<Option<String>> {
        match self.current.token {
            Token::Keyword(ref mut id) => {
                let id = std::mem::take(id);
                self.advance()?;
                Ok(Some(id))
            }
            _ => Ok(None),
        }
    }

    pub fn peek_keyword(&self) -> Result<Option<&str>> {
        match &self.current.token {
            Token::Keyword(id) => {
                Ok(Some(&id))
            }
            _ => Ok(None),
        }
    }
    
    pub fn take_keyword_if(&mut self, pred: fn(&str) -> bool) -> Result<Option<String>> {
        match self.current.token {
            Token::Keyword(ref mut id) if pred(id) => {
                let id = std::mem::take(id);
                self.advance()?;
                Ok(Some(id))
            },
            _ => Ok(None)
        }
    }

    pub fn peek_next_keyword(&self) -> Result<Option<&str>> {
        match &self.next.token {
            Token::Keyword(id) => {
                Ok(Some(&id))
            }
            _ => Ok(None),
        }
    }

    fn try_unsigned(&mut self) -> Result<Option<u64>> {
        match self.current.token {
            Token::Unsigned(ref mut val) => {
                let val = std::mem::take(val);
                self.advance()?;
                Ok(Some(val))
            }
            _ => Ok(None),
        }
    }

    pub fn expect_valtype(&mut self) -> Result<ValueType> {
        self.try_valtype()?
            .ok_or_else(|| error!("expected value type"))
    }

    pub fn try_valtype(&mut self) -> Result<Option<ValueType>> {
        let result = match &self.current.token {
            Token::Keyword(kw) => match kw.as_str() {
                "func" | "funcref" => Some(ValueType::Ref(RefType::Func)),
                "extern" | "externref" => Some(ValueType::Ref(RefType::Extern)),
                "i32" => Some(ValueType::Num(NumType::I32)),
                "i64" => Some(ValueType::Num(NumType::I64)),
                "f32" => Some(ValueType::Num(NumType::F32)),
                "f64" => Some(ValueType::Num(NumType::F64)),
                _ => None,
            },
            _ => None,
        };
        if result.is_some() {
            self.advance()?;
        }
        Ok(result)
    }

    pub fn expect_limits(&mut self) -> Result<Limits> {
        let lower = self.expect_integer()? as u32;
        let upper = self.try_integer()?.map(|l| l as u32);
        Ok(Limits { lower, upper })
    }

    pub fn expect_reftype(&mut self) -> Result<RefType> {
        self.try_reftype()?
            .ok_or_else(|| error!("expected reftype"))
    }

    pub fn try_reftype(&mut self) -> Result<Option<RefType>> {
        let result = match &self.current.token {
            Token::Keyword(kw) => match kw.as_str() {
                "func" | "funcref" => Some(RefType::Func),
                "extern" | "externref" => Some(RefType::Extern),
                _ => None,
            },
            _ => None,
        };
        if result.is_some() {
            self.advance()?;
        }
        Ok(result)
    }

    pub fn try_number(&mut self) -> Result<Option<u64>> {
        match self.current.token {
            Token::Unsigned(val) => {
                self.advance()?;
                Ok(Some(val as u64))
            },
            Token::Signed(val) => {
                self.advance()?;
                Ok(Some(val as u64))
            },
            Token::Float(val) => {
                self.advance()?;
                Ok(Some(val as u64))
            },
            _ => Ok(None)
        }
    }
    
    pub fn expect_number(&mut self) -> Result<u64> {
        self.try_number()?.ok_or_else(|| error!("Expected number token {:?}", self.current.token))
    }

    pub fn try_integer(&mut self) -> Result<Option<u64>> {
        match self.current.token {
            Token::Unsigned(val) => {
                self.advance()?;
                Ok(Some(val as u64))
            },
            Token::Signed(val) => {
                self.advance()?;
                Ok(Some(val as u64))
            },
            _ => Ok(None)
        }
    }

    pub fn expect_integer(&mut self) -> Result<u64> {
        self.try_integer()?.ok_or_else(|| error!("Expected integer token at {:?}", self.current))
    }

    pub fn try_float(&mut self) -> Result<Option<f64>> {
        match self.current.token {
            Token::Float(ref mut val) => {
                let out = std::mem::take(val);
                self.advance()?;
                Ok(Some(out))
            },
            _ => Ok(None)
        }
    }
    
    pub fn expect_float(&mut self) -> Result<f64> {
        self.try_float()?.ok_or_else(|| error!("Expected float token"))
    }

    pub fn consume_expression(&mut self) -> Result<()> {
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

    /// Attempts to parse a series of items using the provided parse method.
    /// The parse method should return 0 or 1 of the item type.
    /// Returns the results as a vector of items.
    pub fn zero_or_more<T>(&mut self, parse: ParseFn<Self, T>) -> Result<Vec<T>> {
        let mut result: Vec<T> = vec![];
        while let Some(t) = parse(self)? {
            result.push(t);
        }
        Ok(result)
    }

    /// Attempts to parse a series of items using the provided parse method.
    /// The parse method should return 0 or more of the item type.
    /// Returns the results as a flattened vector of items.
    pub fn zero_or_more_groups<T>(&mut self, parse: ParseGroupFn<Self, T>) -> Result<Vec<T>> {
        let mut result: Vec<T> = vec![];
        while let Some(t) = parse(self)? {
            result.extend(t);
        }
        Ok(result)
    }

    /// Returns the first successful parse result from the provided list of 
    /// parse methods, otherwise none.
    pub fn first_of<T>(&mut self, parsers: &[ParseFn<Self,T>]) -> Result<Option<T>> {
        for parse in parsers {
            match parse(self) {
                Err(e) => return Err(e),
                Ok(Some(t)) => return Ok(Some(t)),
                Ok(None) => (),
            }
        }
        Ok(None)
    }
}
