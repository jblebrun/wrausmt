use std::io::Read;
use super::Parser;
use super::super::token::Token;
use crate::error::Result;
use crate::error;
use crate::types::Limits;

impl<R: Read> Parser<R> {
    pub fn try_unsigned(&mut self) -> Result<Option<u64>> {
        match self.current.token {
            Token::Unsigned(ref mut val) => {
                let val = std::mem::take(val);
                self.advance()?;
                Ok(Some(val))
            }
            _ => Ok(None),
        }
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

    pub fn expect_limits(&mut self) -> Result<Limits> {
        let lower = self.expect_integer()? as u32;
        let upper = self.try_integer()?.map(|l| l as u32);
        Ok(Limits { lower, upper })
    }
}
