use super::super::token::Token;
use super::error::{ParseError, Result};
use super::Parser;
use crate::{format::text::token::Sign, types::Limits};
use std::io::Read;

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
        if let Some(i) = self.try_integer()? {
            return Ok(Some(i));
        }
        if let Some(f) = self.try_float()? {
            return Ok(Some(f as u64));
        }
        Ok(None)
    }

    pub fn expect_number(&mut self) -> Result<u64> {
        self.try_number()?
            .ok_or_else(|| ParseError::unexpected("number token"))
    }

    pub fn try_integer(&mut self) -> Result<Option<u64>> {
        match self.current.token {
            Token::Unsigned(val) => {
                self.advance()?;
                Ok(Some(val as u64))
            }
            Token::Signed(val) => {
                self.advance()?;
                Ok(Some(val as u64))
            }
            _ => Ok(None),
        }
    }

    pub fn expect_integer(&mut self) -> Result<u64> {
        self.try_number()?
            .ok_or_else(|| ParseError::unexpected("integer token"))
    }

    pub fn try_float(&mut self) -> Result<Option<f64>> {
        match self.current.token {
            Token::Float(ref mut val) => {
                let out = std::mem::take(val);
                self.advance()?;
                Ok(Some(out))
            }
            Token::Inf(sign) => {
                self.advance()?;
                match sign {
                    Sign::Negative => Ok(Some(f64::NEG_INFINITY)),
                    _ => Ok(Some(f64::INFINITY)),
                }
            }
            Token::NaNx(_, _) | Token::NaN(_) => {
                self.advance()?;
                Ok(Some(f64::NAN))
            }
            _ => Ok(None),
        }
    }

    pub fn expect_float(&mut self) -> Result<f64> {
        self.try_float()?
            .ok_or_else(|| ParseError::unexpected("float token"))
    }

    pub fn expect_limits(&mut self) -> Result<Limits> {
        let lower = self.expect_integer()? as u32;
        let upper = self.try_integer()?.map(|l| l as u32);
        Ok(Limits { lower, upper })
    }
}
