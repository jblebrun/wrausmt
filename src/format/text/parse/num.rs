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
    ( $name:ident, $ty:ty, $nanx:ident, $hex:ident ) => {
        pub fn $name(&self) -> Result<$ty> {
            println!("ATTEMPT FLOAT {:?}", self);
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
                NumToken::Float(sign, base, whole, frac, exp) => {
                    let exp = if exp.is_empty() { "0" } else { exp };
                    match base {
                        Base::Dec => {
                            let to_parse = format!("{}{}.{}e{}", sign.char(), whole, frac, exp);
                            println!("PARSE DECIMAL FLOAT {}", to_parse);
                            Ok(to_parse.parse::<$ty>()?)
                        }
                        Base::Hex => $hex(*sign, whole, frac, exp),
                    }
                }
                NumToken::Integer(sign, base, digits) => match base {
                    Base::Dec => {
                        let to_parse = format!("{}{}", sign.char(), digits);
                        Ok(to_parse.parse::<$ty>()?)
                    }
                    Base::Hex => {
                        println!("TRY INT AS HEX {}", digits);
                        $hex(*sign, digits, "", "0")
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
    parse_float! { as_f32, f32, nanx_f32, parse_hex_f32 }
    parse_float! { as_f64, f64, nanx_f64, parse_hex_f64  }
}

fn parse_hex_digit(digit_byte: u8) -> Result<u8> {
    match digit_byte {
        c @ b'a'..=b'f' => Ok(c - b'a' + 10),
        c @ b'A'..=b'F' => Ok(c - b'A' + 10),
        c @ b'0'..=b'9' => Ok(c - b'0'),
        _ => Err(ParseError::unexpected("hex digit")),
    }
}

#[derive(Debug, Default)]
struct Mantissa {
    bits: u64,
}

impl Mantissa {
    fn add_digit(&mut self, dbyte: u8) -> Result<()> {
        let d = parse_hex_digit(dbyte)?;
        let round_bits = self.bits & 0xF;
        self.bits >>= 4;
        self.bits |= (d as u64) << 60;
        // The exact value in the lower nybble doesn't matter,
        // we just need for it to be non-zero if any bits shifting out
        // will have triggered rounding up.
        self.bits |= round_bits;
        Ok(())
    }

    fn normalize(&mut self) -> u32 {
        let lz = self.bits.leading_zeros();
        self.bits <<= lz + 1;
        lz
    }

    /// Handle round-to-nearest for a mantissa of the provided size.
    fn round(&mut self, size: u32) {
        let mask = u64::MAX >> size;
        let round_part = self.bits & mask;
        let mantissa_lsb = self.bits >> (64 - size) & 0x1;
        let round_max = 1 << (64 - size - 1);
        let round = round_part > round_max || round_part == round_max && mantissa_lsb == 1;
        self.bits >>= 64 - size;
        if round {
            self.bits += 1
        }
    }
}

/// Return mantissa bits, exp offset
/// Returns a 64 bit mantissa with enough information
/// to properly round both f32 and f64
fn parse_mantissa_64(whole: &str, frac: &str) -> Result<(Mantissa, i16)> {
    // Consume meaningless 0s
    let whole = whole.trim_start_matches('0');
    let frac = frac.trim_end_matches('0');

    let mut mantissa = Mantissa::default();

    // Shift all fractional bytes in through the top.
    for frac_byte in frac.bytes().rev() {
        mantissa.add_digit(frac_byte)?;
    }

    let exp_offset = if !whole.is_empty() {
        // While handling whole digits, we may go supernormal,
        // so track the exponent offset here.
        let mut exp_offset = 0i16;
        for whole_byte in whole.bytes().rev() {
            mantissa.add_digit(whole_byte)?;
            exp_offset += 4;
        }

        let normalize_offset = mantissa.normalize();
        exp_offset -= normalize_offset as i16 + 1;
        exp_offset
    } else {
        0
    };

    println!("MANTISSA: {:016x}, {}", mantissa.bits, exp_offset);

    Ok((mantissa, exp_offset))
}

fn parse_hex_f32(sign: Sign, whole: &str, frac: &str, exp: &str) -> Result<f32> {
    println!("PARSE HEX F32 {:?} {} {} {}", sign, whole, frac, exp);
    let exp = exp.parse::<i16>()?;

    let (mut mantissa, exp_offset) = parse_mantissa_64(whole, frac)?;

    mantissa.round(f32::MANTISSA_DIGITS - 1);

    let mut result_bits = mantissa.bits as u32;

    let exp = exp + exp_offset;

    if exp > 127 || exp < -127 {
        return Err(ParseError::unexpected("f32"));
    }

    let offset_exp = (exp + 127) as u32;
    result_bits |= offset_exp << 23;

    if sign == Sign::Negative {
        result_bits |= 0x80000000;
    }

    println!("f32 bits: {:x}", result_bits);
    Ok(f32::from_bits(result_bits))
}

fn parse_hex_f64(sign: Sign, whole: &str, frac: &str, exp: &str) -> Result<f64> {
    println!("PARSE HEX F64 {:?} {} {} {}", sign, whole, frac, exp);
    let exp = exp.parse::<i16>()?;

    let (mut mantissa, exp_offset) = parse_mantissa_64(whole, frac)?;

    mantissa.round(f64::MANTISSA_DIGITS - 1);

    let mut result_bits = mantissa.bits;

    let exp = exp + exp_offset;

    if exp > 1023 || exp < -1022 {
        return Err(ParseError::unexpected("f64"));
    }

    let offset_exp = (exp + 1023) as u64;
    result_bits |= offset_exp << 52;

    if sign == Sign::Negative {
        result_bits |= 0x8000000000000000;
    }

    println!("f64 bits: {:x}", result_bits);
    Ok(f64::from_bits(result_bits))
}

#[cfg(test)]
mod tests {
    use super::super::super::token::Sign;
    use super::parse_hex_f32;
    use super::parse_hex_f64;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn hex64_normal() -> Result<()> {
        let result = parse_hex_f64(Sign::Unspecified, "145", "23", "12")?;
        assert_eq!(result, (0x14523 as f64) * 2f64.powi(4));

        let result = parse_hex_f64(Sign::Negative, "1", "0000000000002", "60")?;
        assert_eq!(result, f64::from_bits(0xc3b0000000000002));

        let result = parse_hex_f64(Sign::Unspecified, "123456789abcdef", "23", "12")?;
        assert_eq!(result, (0x123456789abcdfu64 as f64) * 2f64.powi(16));

        Ok(())
    }

    #[test]
    fn hex32_normal() -> Result<()> {
        let result = parse_hex_f32(Sign::Unspecified, "145", "23", "12")?;
        assert_eq!(result, (0x14523 as f32) * 2f32.powi(4));

        let result = parse_hex_f32(Sign::Unspecified, "1", "000001fffffffffff", "-50")?;
        assert_eq!(result, f32::from_bits(0x26800001));

        let result = parse_hex_f32(Sign::Unspecified, "1", "000002", "-50")?;
        assert_eq!(result, f32::from_bits(0x26800001));
        Ok(())
    }
}
