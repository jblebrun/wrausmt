use super::Token;
use crate::format::text::token::{Base, NumToken, Sign};

fn is_digit(ch: char, hex: bool) -> bool {
    if matches!(ch, '0'..='9' | '_') {
        return true;
    }
    if !hex {
        return false;
    }

    matches!(ch, 'a'..='f' | 'A'..='F')
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

    pub fn consume_while(&mut self, pred: impl Fn(char) -> bool) -> String {
        let mut i = 0;
        let mut iter = self.chars.chars();
        while matches!(iter.next(), Some(cur) if pred(cur)) {
            i += 1;
        }

        let result = self.chars.get(0..i).unwrap();

        self.advance_by(i);
        result.to_owned()
    }
}

/// Attempt to interpret the `idchars` as a number. If the conversion is successful, a [Token] is
/// returned, otherwise, [None] is returned.
pub fn maybe_number(idchars: &str) -> Option<Token> {
    let mut cursor = StrCursor::new(idchars);

    let sign: Sign = cursor.cur().unwrap().into();

    if sign != Sign::Unspecified {
        cursor.advance_by(1);
    }

    if cursor.is_exactly("nan") {
        return Some(Token::Number(NumToken::NaN(sign)));
    }

    if cursor.is_exactly("inf") {
        return Some(Token::Number(NumToken::Inf(sign)));
    }

    if cursor.matches("nan:0x") {
        cursor.advance_by(6);
        let payload = cursor.consume_while(|c| is_digit(c, true));
        return match cursor.is_empty() {
            true => Some(Token::Number(NumToken::NaNx(sign, payload))),
            false => None,
        };
    }

    let hex = cursor.matches("0x");
    if hex {
        cursor.advance_by(2);
    }

    let base = if hex { Base::Hex } else { Base::Dec };

    let whole = cursor.consume_while(|c| is_digit(c, hex));
    let whole = whole.replace('_', "");

    if whole.is_empty() {
        return None;
    }

    let have_point = cursor.matches(".");
    if have_point {
        cursor.advance_by(1)
    }

    let frac = cursor.consume_while(|c| is_digit(c, hex));
    let frac = frac.replace('_', "");

    let expchars = if hex { ['p', 'P'] } else { ['e', 'E'] };

    let have_exp = matches!(cursor.cur(), Some(cur) if expchars.contains(&cur));

    if have_exp {
        cursor.advance_by(1);
    }

    let exp = cursor.consume_while(|c| is_digit(c, false) || c == '+' || c == '-');
    let exp = exp.replace('_', "");

    if !cursor.is_empty() {
        return None;
    }

    if have_point || have_exp {
        Some(Token::Number(NumToken::Float(sign, base, whole, frac, exp)))
    } else {
        Some(Token::Number(NumToken::Integer(sign, base, whole)))
    }
}
