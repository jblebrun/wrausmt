use crate::text::token::{Base, NumToken, Sign};

fn is_digit(ch: u8, base: Base) -> bool {
    ch.is_ascii_digit() || (matches!(base, Base::Hex) && matches!(ch, b'a'..=b'f' | b'A'..=b'F'))
}

#[derive(Debug)]
struct StrCursor<'a> {
    chars: &'a str,
}

impl<'a> StrCursor<'a> {
    pub fn new(chars: &'a str) -> Self {
        Self { chars }
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    pub fn cur(&self) -> Option<char> {
        self.chars.chars().next()
    }

    pub fn advance_by(&mut self, amt: usize) {
        let (_, rest) = self.chars.split_at(amt);
        self.chars = rest;
    }

    pub fn matches(&self, other: &str) -> bool {
        self.chars.starts_with(other)
    }

    pub fn is_exactly(&self, other: &str) -> bool {
        self.chars == other
    }

    // Consume a group of digits, which can be separated by _
    // But no _ before or after, and no contiguous.
    pub fn consume_digit_group(&mut self, base: Base) -> String {
        let mut i = 0;
        let bytes = self.chars.as_bytes();
        let mut result = String::new();
        let mut final_sep = false;

        while i < bytes.len() && is_digit(bytes[i], base) {
            final_sep = false;
            result.push(bytes[i] as char);
            i += 1;
            if i < bytes.len() && bytes[i] == b'_' {
                final_sep = true;
                i += 1;
            }
        }

        if final_sep {
            String::new()
        } else {
            self.advance_by(i);
            result
        }
    }
}

/// Attempt to interpret the `idchars` as a number. If the conversion is
/// successful, a [NumToken] is returned, otherwise, [None] is returned.
pub fn maybe_number(idchars: &str) -> Option<NumToken> {
    let mut cursor = StrCursor::new(idchars);

    let sign: Sign = cursor.cur().unwrap().into();

    if sign != Sign::Unspecified {
        cursor.advance_by(1);
    }

    if cursor.is_exactly("nan") {
        return Some(NumToken::NaN(sign));
    }

    if cursor.is_exactly("inf") {
        return Some(NumToken::Inf(sign));
    }

    if cursor.matches("nan:0x") {
        cursor.advance_by(6);
        let payload = cursor.consume_digit_group(Base::Hex);
        return match cursor.is_empty() {
            true => Some(NumToken::NaNx(sign, payload)),
            false => None,
        };
    }

    let base = if cursor.matches("0x") {
        cursor.advance_by(2);
        Base::Hex
    } else {
        Base::Dec
    };

    let whole = cursor.consume_digit_group(base);

    if whole.is_empty() {
        return None;
    }

    let have_point = cursor.matches(".");
    let frac = if have_point {
        cursor.advance_by(1);
        cursor.consume_digit_group(base)
    } else {
        String::new()
    };

    let expchars = match base {
        Base::Hex => ['p', 'P'],
        Base::Dec => ['e', 'E'],
    };

    let have_exp = matches!(cursor.cur(), Some(cur) if expchars.contains(&cur));

    if have_exp {
        cursor.advance_by(1);
    }

    let mut exp = match cursor.cur() {
        Some('+') => "+".to_string(),
        Some('-') => "-".to_string(),
        _ => String::new(),
    };

    cursor.advance_by(exp.len());

    exp += &cursor.consume_digit_group(Base::Dec);

    if !cursor.is_empty() {
        return None;
    }

    if have_point || have_exp {
        Some(NumToken::Float(sign, base, whole, frac, exp))
    } else {
        Some(NumToken::Integer(sign, base, whole))
    }
}
