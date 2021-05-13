use super::super::token::{Base, NumToken, Sign, Token};
use super::error::{ParseError, Result};
use super::Parser;
use crate::types::Limits;
use std::io::Read;

macro_rules! try_num {
    ( $n:ident, $en:ident, $fn:ident, $ty:ty, $err:literal ) => {
        pub fn $n(&mut self) -> Result<Option<$ty>> {
            match &self.current.token {
                Token::Number(numtoken) => {
                    let val = numtoken.$fn()?;
                    self.advance()?;
                    Ok(Some(val))
                }
                _ => Ok(None),
            }
        }

        pub fn $en(&mut self) -> Result<$ty> {
            let got = self.$n()?;
            match got {
                Some(v) => Ok(v),
                None => Err(ParseError::unexpected($err)),
            }
        }
    };
}

impl<R: Read> Parser<R> {
    try_num! { try_u32, expect_u32, as_u32, u32, "U32" }
    try_num! { try_i32, expect_i32, as_i32, i32, "I32" }
    try_num! { try_i64, expect_i64, as_i64, i64, "I64" }
    try_num! { try_f32, expect_f32, as_f32, f32, "F32" }
    try_num! { try_f64, expect_f64, as_f64, f64, "F64" }

    pub fn expect_limits(&mut self) -> Result<Limits> {
        let lower = self.expect_u32()?;
        let upper = self.try_u32()?;
        Ok(Limits { lower, upper })
    }
}

fn nanx_f64(sign: Sign, _payload: &str) -> Result<f64> {
    let base: u64 = match sign {
        Sign::Negative => 0xFFF8000000000000,
        _ => 0x7FF8000000000000,
    };
    Ok(<f64>::from_bits(base))
}

fn nanx_f32(sign: Sign, _payload: &str) -> Result<f32> {
    let base: u32 = match sign {
        Sign::Negative => 0xFFC00000,
        _ => 0x7FC00000,
    };
    Ok(<f32>::from_bits(base))
}

macro_rules! parse_float {
    ( $name:ident, $ty:ty, $nanx:ident ) => {
        pub fn $name(&self) -> Result<$ty> {
            match self {
                NumToken::NaN(sign) => match sign {
                    Sign::Negative => Ok(-<$ty>::NAN),
                    _ => Ok(<$ty>::NAN),
                },
                NumToken::NaNx(sign, payload) => $nanx(*sign, payload),
                NumToken::Inf(sign) => match sign {
                    Sign::Negative => Ok(<$ty>::NEG_INFINITY),
                    _ => Ok(<$ty>::INFINITY),
                },
                NumToken::Float(sign, base, whole, frac, exp) => match base {
                    Base::Dec => {
                        let expchar = if exp.is_empty() { "" } else { "e" };
                        let to_parse =
                            format!("{}{}.{}{}{}", sign.char(), whole, frac, expchar, exp);
                        println!("PARSE FLOAT {}", to_parse);
                        Ok(to_parse.parse::<$ty>()?)
                    }
                    Base::Hex => Ok(0.0),
                },
                NumToken::Integer(sign, base, digits) => match sign {
                    Sign::Negative => {
                        let to_parse = format!("{}{}", sign.char(), digits);
                        Ok(<i64>::from_str_radix(&to_parse, base.radix())? as $ty)
                    }
                    _ => {
                        let to_parse = format!("{}{}", sign.char(), digits);
                        Ok(<u64>::from_str_radix(&to_parse, base.radix())? as $ty)
                    }
                },
            }
        }
    };
}

macro_rules! parse_int {
    ( $name:ident, $ty:ty, $uty:ty, $err:literal ) => {
        pub fn $name(&self) -> Result<$ty> {
            match self {
                NumToken::Integer(sign, base, digits) => match sign {
                    Sign::Negative => {
                        let to_parse = format!("{}{}", sign.char(), digits);
                        Ok(<$ty>::from_str_radix(&to_parse, base.radix())?)
                    }
                    _ => {
                        let to_parse = format!("{}{}", sign.char(), digits);
                        Ok(<$uty>::from_str_radix(&to_parse, base.radix())? as $ty)
                    }
                },
                _ => Err(ParseError::unexpected($err)),
            }
        }
    };
}

impl NumToken {
    parse_int! { as_u32, u32, u32, "u32" }
    parse_int! { as_i32, i32, u32, "i32" }
    parse_int! { as_i64, i64, u64, "i64" }
    parse_float! { as_f32, f32, nanx_f32 }
    parse_float! { as_f64, f64, nanx_f64 }
}
