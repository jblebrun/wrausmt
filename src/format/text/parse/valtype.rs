use crate::types::{NumType, RefType, ValueType};
use std::io::Read;
use super::Parser;
use super::error::{ParseError, Result};
use super::super::token::Token;

impl<R: Read> Parser<R> {
    pub fn expect_valtype(&mut self) -> Result<ValueType> {
        self.try_valtype()?
            .ok_or_else(|| ParseError::unexpected("value type"))
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

    pub fn expect_reftype(&mut self) -> Result<RefType> {
        self.try_reftype()?
            .ok_or_else(|| ParseError::unexpected("value type"))
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
}
