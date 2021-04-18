use super::Token;
use crate::format::text::token::Sign;

/// Interpret a discriminated nan, "nan:0xABCDE".
fn maybe_nan_or_inf(numchars: &str, sign: Sign) -> Option<Token> {
    match numchars {
        nc if nc == "nan" => Some(Token::NaN(sign)),
        nc if nc == "inf" => Some(Token::Inf(sign)),
        nc if nc.starts_with("nan:0x") => {
            let (_, numpart) = &numchars.split_at(6);
            match u32::from_str_radix(numpart, 16) {
                Ok(nanx) => Some(Token::NaNx(sign, nanx as u32)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Attempt to interpret the `idchars` as a number. If the conversion is successful, a [Token] is
/// returned, otherwise, [None] is returned.
pub fn maybe_number(idchars: &str) -> Option<Token> {
    let snumchars = idchars.replace("_", "");
    let mut numchars = snumchars.as_str();

    if numchars.is_empty() {
        return None;
    }

    let sign: Sign = numchars.chars().next().unwrap().into();
    if sign != Sign::Unspecified {
        let (_, rest) = numchars.split_at(1);
        numchars = rest;
    }

    if let Some(t) = maybe_nan_or_inf(numchars, sign) {
        return Some(t);
    }

    let radix = match numchars {
        bs if bs.starts_with("0x") => {
            let (_, rest) = numchars.split_at(2);
            numchars = rest;
            16
        }
        _ => 10,
    };

    // Now try integer
    if let Ok(val) = u64::from_str_radix(numchars, radix) {
        return Some(match sign {
            Sign::Unspecified => Token::Unsigned(val),
            Sign::Positive => Token::Signed(val as i64),
            Sign::Negative => {
                let iv = val as i64;
                if iv < 0 {
                    Token::Signed(iv)
                } else {
                    Token::Signed(-iv)
                }
            }
        });
    }

    // Float w/o leading zero not accepted.
    if numchars.as_bytes()[0] == b'.' {
        return None;
    }

    // Now try decimal float:
    if let Ok(val) = numchars.parse::<f64>() {
        return match sign {
            Sign::Positive | Sign::Unspecified => Some(Token::Float(val)),
            Sign::Negative => Some(Token::Float(-val)),
        };
    }

    // TODO hex float
    None
}
