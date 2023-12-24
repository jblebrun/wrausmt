use {
    super::{
        super::token::{Base, NumToken, Sign, Token},
        error::{KindResult, ParseErrorKind, Result},
        ParseResult, Parser,
    },
    crate::syntax::types::Limits,
    std::io::Read,
};

macro_rules! try_num {
    ( $n:ident, $en:ident, $fn:ident, $ty:ty, $err:literal ) => {
        pub fn $n(&mut self) -> Result<Option<$ty>> {
            match &self.current.token {
                Token::Number(numtoken) => {
                    let val = numtoken.$fn().result(self)?;
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
                None => Err(self.unexpected_token($err)),
            }
        }
    };
}

impl<R: Read> Parser<R> {
    try_num! { try_u32, expect_u32, as_u32, u32, "expected U32" }

    try_num! { try_i32, expect_i32, as_i32, i32, "expected I32" }

    try_num! { try_i64, expect_i64, as_i64, i64, "expected I64" }

    try_num! { try_f32, expect_f32, as_f32, f32, "expected F32" }

    try_num! { try_f64, expect_f64, as_f64, f64, "expected F64" }

    pub fn expect_limits(&mut self) -> Result<Limits> {
        let lower = self.expect_u32()?;
        let upper = self.try_u32()?;
        Ok(Limits { lower, upper })
    }
}

fn nanx_f64(sign: Sign, payload: &str) -> KindResult<f64> {
    let payload = payload.replace('_', "");
    let payload_num = u64::from_str_radix(&payload, 16)?;
    let base: u64 = match sign {
        Sign::Negative => 0xFFF0000000000000,
        _ => 0x7FF0000000000000,
    };
    Ok(<f64>::from_bits(base | payload_num))
}

fn nanx_f32(sign: Sign, payload: &str) -> KindResult<f32> {
    let payload = payload.replace('_', "");
    let payload_num = u32::from_str_radix(&payload, 16)?;
    let base: u32 = match sign {
        Sign::Negative => 0xFF800000,
        _ => 0x7F800000,
    };
    Ok(<f32>::from_bits(base | payload_num))
}

macro_rules! parse_float {
    ( $name:ident, $ty:ty, $nanx:ident, $hex:ident ) => {
        pub fn $name(&self) -> KindResult<$ty> {
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
                    Base::Hex => $hex(*sign, digits, "", "0"),
                },
            }
        }
    };
}

macro_rules! parse_int {
    ( $name:ident, $ty:ty, $uty:ty, $err:literal ) => {
        pub fn $name(&self) -> KindResult<$ty> {
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
                _ => Err(ParseErrorKind::UnexpectedToken($err.into())),
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

fn parse_hex_digit(digit_byte: u8) -> KindResult<u8> {
    match digit_byte {
        c @ b'a'..=b'f' => Ok(c - b'a' + 10),
        c @ b'A'..=b'F' => Ok(c - b'A' + 10),
        c @ b'0'..=b'9' => Ok(c - b'0'),
        _ => Err(ParseErrorKind::UnexpectedToken("expected hex digit".into())),
    }
}

/// A helper structure for aggregating the mantissa and adjusting the exponent.
/// It carries enough information to eventually round a mantissa that's 60 bits
/// or less. Can generate either f32 or f64.
#[derive(Debug, Default)]
struct FloatBuilder {
    bits: u64,
    exp:  i16,
}

impl FloatBuilder {
    fn new(whole: &str, frac: &str, exp: &str) -> KindResult<Self> {
        // Consume meaningless 0s
        let whole = whole.trim_start_matches('0');
        let frac = frac.trim_end_matches('0');
        let exp = exp.parse::<i16>()?;

        let mut builder = FloatBuilder { bits: 0, exp };

        // Shift all fractional bytes in through the top.
        for frac_byte in frac.bytes().rev() {
            builder.add_frac_digit(frac_byte)?;
        }

        // Shift all whole bytes in through the top.
        for whole_byte in whole.bytes().rev() {
            builder.add_whole_digit(whole_byte)?;
        }

        builder.normalize();

        Ok(builder)
    }

    /// Add a digit to the mantissa. Digits should be added most-significant
    /// first.
    fn add_digit(&mut self, dbyte: u8) -> KindResult<()> {
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

    fn add_frac_digit(&mut self, dbyte: u8) -> KindResult<()> {
        self.add_digit(dbyte)
    }

    fn add_whole_digit(&mut self, dbyte: u8) -> KindResult<()> {
        self.exp += 4;
        self.add_digit(dbyte)
    }

    /// Normalize the mantissa. This should be called once all digits have been
    /// shifted in, if the mantissa is for a normal number (there are any non-0
    /// whole number digits).
    fn normalize(&mut self) {
        if self.bits == 0 {
            return;
        }
        let lz = self.bits.leading_zeros();
        self.bits <<= lz;
        self.exp -= lz as i16;
    }

    /// Handle round-to-nearest, ties-to-even for a mantissa of the provided
    /// size. Round up when the out-of-range digits are more than half LSB,
    /// Round down when out-of-range digits are less than half LSB,
    /// When out-of-range digits are exactly half, LSB, round to nearest even,
    /// i.e.:   round up when MSB 1, down when MSB 0.
    fn round(&mut self, size: u32) {
        let roundmask = u64::MAX >> size;
        let even = 0x8000000000000000u64 >> size;
        let roundpart = self.bits & roundmask;
        let mantissa_lsb = self.bits & even << 1;
        let round = roundpart > even || roundpart == even && mantissa_lsb != 0;

        self.bits >>= 64 - size;

        if round {
            self.bits += 1;
        }
    }

    /// Adjust the mantissa so that the exp is in range.
    fn range(&mut self, minexp: i16) {
        if self.exp <= minexp + 1 {
            self.bits >>= 1;
            while self.exp <= minexp {
                // When ranging, also "gather" bits in the least significant nybble for
                // rounding purposes.
                let round_bits = self.bits & 0xF;
                self.bits >>= 1;
                self.bits |= round_bits;
                self.exp += 1;
            }
        }
        self.exp -= 1;
    }

    fn build(mut self, mantissa_size: u32, expmax: u32) -> KindResult<u64> {
        self.range(-(expmax as i16));
        self.round(mantissa_size);

        if self.bits == 0 {
            return Ok(0);
        }

        if self.exp > expmax as i16 {
            return Err(ParseErrorKind::UnexpectedToken("floatrange".into()));
        }

        let mask = u64::MAX >> (64 - mantissa_size + 1);
        self.bits &= mask;

        let offset_exp = (self.exp + expmax as i16) as u64;
        self.bits |= offset_exp << (mantissa_size - 1);

        Ok(self.bits)
    }
}

fn parse_hex_f32(sign: Sign, whole: &str, frac: &str, exp: &str) -> KindResult<f32> {
    let builder = FloatBuilder::new(whole, frac, exp)?;

    let mut result_bits = builder.build(f32::MANTISSA_DIGITS, 127)? as u32;

    if sign == Sign::Negative {
        result_bits |= 0x80000000;
    }

    Ok(f32::from_bits(result_bits))
}

fn parse_hex_f64(sign: Sign, whole: &str, frac: &str, exp: &str) -> KindResult<f64> {
    let builder = FloatBuilder::new(whole, frac, exp)?;

    let mut result_bits = builder.build(f64::MANTISSA_DIGITS, 1023)?;

    if sign == Sign::Negative {
        result_bits |= 0x8000000000000000;
    }

    Ok(f64::from_bits(result_bits))
}

#[cfg(test)]
mod tests {
    use {
        super::{super::super::token::Sign, parse_hex_f32, parse_hex_f64},
        crate::format::text::parse::error::ParseErrorKind,
    };
    impl std::error::Error for ParseErrorKind {}

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
        let result = parse_hex_f32(Sign::Unspecified, "1", "", "-1")?;
        assert_eq!(result, 0.5);

        let result = parse_hex_f32(Sign::Unspecified, "0", "8", "0")?;
        assert_eq!(result, 0.5);

        let result = parse_hex_f32(Sign::Unspecified, "0", "4", "1")?;
        assert_eq!(result, 0.5);

        let result = parse_hex_f32(Sign::Unspecified, "145", "23", "12")?;
        assert_eq!(result, (0x14523 as f32) * 2f32.powi(4));

        let result = parse_hex_f32(Sign::Unspecified, "1", "000002", "-50")?;
        assert_eq!(result, f32::from_bits(0x26800001));

        let result = parse_hex_f32(Sign::Unspecified, "1", "000001fffffffffff", "-50")?;
        assert_eq!(result, f32::from_bits(0x26800001));

        let result = parse_hex_f32(Sign::Unspecified, "0", "0", "0")?;
        assert_eq!(result, 0f32);
        Ok(())
    }

    #[test]
    fn hex32_subnormal() -> Result<()> {
        let result = parse_hex_f32(Sign::Unspecified, "1", "", "-149")?;
        assert_eq!(result.to_bits(), 1);

        let result = parse_hex_f32(Sign::Unspecified, "0", "8", "-148")?;
        println!("HEX32SN BITS {:08x}", result.to_bits());
        assert_eq!(result.to_bits(), 1);

        let result = parse_hex_f32(Sign::Unspecified, "0", "000001", "-126")?;
        println!("HEX32SN BITS {:08x}", result.to_bits());
        assert_eq!(result.to_bits(), 0);

        let result = parse_hex_f32(Sign::Unspecified, "1", "fffffc", "-127")?;
        println!("HEX32SN BITS {:08x}", result.to_bits());
        assert_eq!(result.to_bits(), 0x7fffff);

        let result = parse_hex_f32(Sign::Unspecified, "0", "00000100000000000", "-126")?;
        println!("HEX32SN BITS {:08x}", result.to_bits());
        assert_eq!(result.to_bits(), 0);

        let result = parse_hex_f32(Sign::Unspecified, "0", "00000100000000001", "-126")?;
        println!("HEX32SN BITS {:08x}", result.to_bits());
        assert_eq!(result.to_bits(), 1);

        let result = parse_hex_f32(Sign::Unspecified, "0", "000000", "-126")?;
        println!("HEX32SN BITS {:08x}", result.to_bits());
        assert_eq!(result.to_bits(), 0);

        Ok(())
    }

    #[test]
    fn hex64_subnormal() -> Result<()> {
        let result = parse_hex_f64(Sign::Unspecified, "1", "", "-1074")?;
        println!("HEX64SN BITS {:016x}", result.to_bits());
        assert_eq!(result.to_bits(), 1);

        let result = parse_hex_f64(Sign::Unspecified, "0", "0000000000001", "-1022")?;
        println!("HEX64SN BITS {:016x}", result.to_bits());
        assert_eq!(result.to_bits(), 1);

        Ok(())
    }
}
