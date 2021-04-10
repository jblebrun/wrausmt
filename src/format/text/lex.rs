use std::io::Read;
use std::iter::Iterator;
use crate::error::{Error, Result, ResultFrom};
use super::token::Token;

#[derive(Debug)]
pub struct Tokenizer<R> {
    inner: R,
    current: u8,
    eof: bool,
    line: u32,
    pos: u32,
}

impl <R : Read> Tokenizer<R> {
    pub fn new(r: R) -> Result<Tokenizer<R>> {
        let mut tokenizer = Tokenizer {
            inner: r,
            current: 0,
            eof: false,
            line: 0,
            pos: 0
        };
        tokenizer.advance()?;
        Ok(tokenizer)
    }

    fn advance(&mut self) -> Result<()> {
        let mut buf = [0u8; 1];
        let amount_read = self.inner.read(&mut buf).wrap("reading")?;
        self.current = buf[0];
        if amount_read == 0 {
            if self.eof {
                return Err(Error::new("unexpected eof".to_string()))
            } else {
                self.eof = true;
            }
        } else if self.current == b'\n' {
            self.line += 1;
            self.pos = 0;
        } else {
            self.pos += 1;
        }
        Ok(())
    }

    fn is_whitespace(&self) -> bool  {
        matches!(self.current, b' ' | b'\t' | b'\n' | b'\r')
    }

    fn is_idchar(&self) -> bool {
        matches!(self.current, 
            b'0'..=b'9' | b'A'..=b'Z'  | b'a'..=b'z' | b'!' | b'#' |
            b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'/'  |
            b':' | b'<' | b'=' | b'>'  | b'?' | b'@' | b'\\' | 
            b'^' | b'_' | b'`' | b'|'  | b'~' | b'.'
        )
    }

    fn is_digit(&self, hex: bool) -> bool {
        match self.current {
            b'_' | b'0'..=b'9' => true,
            b'a'..=b'f' | b'A'..=b'F' => hex,
            _ => false
        }
    }

    fn is_exp(&self, hex: bool) -> bool {
        match self.current {
            b'e' | b'E' => !hex,
            b'p' | b'P' => hex,
            _ => false
        }
    }

    fn consume_whitespace(&mut self) -> Result<Token> {
        while self.is_whitespace() {
            self.advance()?
        }
        Ok(Token::Whitespace)
    }


    fn consume_line_comment(&mut self) -> Result<Token> {
        while self.current != b'\n' {
            self.advance()?;
        }
        self.advance()?;
        Ok(Token::LineComment)
    }

    // Caller will have consume (, and we will be on the ;
    fn consume_block_comment(&mut self) -> Result<Token> {
        let mut depth = 1;
        self.advance()?;
        while depth > 0 {
            match self.current {
                b'(' => {
                    self.advance()?;
                    if self.current == b';' {
                        depth += 1;
                    }
                },
                b';' => {
                    self.advance()?;
                    if self.current == b')' {
                        depth -=1;
                        if depth == 0 {
                            self.advance()?;
                            break;
                        }
                    }
                },
                _ =>  self.advance()?
            }
        }
        Ok(Token::BlockComment)
    }

    fn consume_string(&mut self) -> Result<Token> {
        let mut result: Vec<u8> = vec![];
        let mut prev: u8 = 0;

        loop {
            self.advance()?;
            if self.current == b'"' && prev != b'\\' {
                let as_string = String::from_utf8(result).wrap("bad utf8")?;
                self.advance()?;
                return Ok(Token::String(as_string))
            }
            result.push(self.current);
            prev = self.current;
        }
    }

    fn consume_other(&mut self) -> Result<Token> {
        Err(Error::new("UNKNOWN".to_string())).wrap("")
    }

    fn consume_name(&mut self) -> Result<String> {
        let mut result: Vec<u8> = vec![];
        while self.is_idchar() {
            result.push(self.current);
            self.advance()?;
        }
        if !self.eof && !self.is_whitespace() && self.current != b')' {
            return Err(Error::new(format!("Invalid char {}", self.current)));
        }
        let sresult = String::from_utf8(result).wrap("bad utf8")?;
        Ok(sresult)
    }

    fn consume_keyword(&mut self) -> Result<Token> {
        let result = self.consume_name()?;
        if result == "nan" {
           return Ok(Token::NaN);
        }
        if result == "inf" {
            return Ok(Token::Inf);
        }
        if result.len() >5 && &result[0..6] == "nan:0x" {
            let nanx = Self::interpret_whole_number(true, result[6..].into());
            return Ok(Token::NaNx(nanx as u32))

        }
        Ok(Token::Keyword(result))
    }

    fn consume_id(&mut self) -> Result<Token> {
        self.advance()?;
        let result = self.consume_name()?;
        Ok(Token::Id(result))
    }

    fn consume_digits(&mut self, hex: bool) -> Result<Vec<u8>> {
        let mut digit_bytes: Vec<u8> = vec![];
        while self.is_digit(hex) {
            digit_bytes.push(self.current);
            self.advance()?;
        }
        Ok(digit_bytes)

    }

    fn digit_val(digit_byte: u8) -> u8 {
        match digit_byte {
            b'0'..=b'9' => digit_byte - b'0',
            b'a'..=b'f' => 10u8 + digit_byte - b'a',
            b'A'..=b'F' => 10u8 + digit_byte - b'A',
            _ => panic!("invalid digit")
        }
    }

    fn interpret_whole_number(hex: bool, digit_bytes: Vec<u8>) -> u64 {
        let mut exp = 1u64;
        let mut result = 0u64;
        let exp_mult = if hex { 16 } else { 10 };
        for digit in digit_bytes.into_iter().rev() {
            if digit != b'_' {
                result += exp * Self::digit_val(digit) as u64;
                exp *= exp_mult;
            }
        }
        result
    }

    fn parse_whole_number(&mut self, hex: bool) -> Result<u64> {
        let digit_bytes = self.consume_digits(hex)?;
        Ok(Self::interpret_whole_number(hex, digit_bytes))
   
    }

    fn parse_frac_number(&mut self, hex: bool) -> Result<f64> {
        let digit_bytes = self.consume_digits(hex)?;
        let mut exp = if hex { 16u64 } else { 10u64 };
        let mut result = 0f64;
        let exp_mult = if hex { 16 } else { 10 };
        for digit in digit_bytes.into_iter().rev() {
            if digit == b'_' { continue; }
            let digit_val = Self::digit_val(digit);
            result += (digit_val) as f64 / exp as f64;
            exp *= exp_mult;
        }
        Ok(result)
    }

    fn sign_info(&mut self) -> (bool, i64) {
        match self.current {
            b'+'  => (true, 1),
            b'-'  => (true, -1),
            _ => (false, 1)
        }
    }

    fn consume_number(&mut self) -> Result<Token> {
        // read the sign if present
        if self.current == b'+' {
            self.advance()?;
        }

        let (signed, sign) = self.sign_info();

        if signed {
            self.advance()?;
        }

        // if 0 check for x
        let hex = if self.current == b'0' {
            self.advance()?;
            if self.current == b'x' {
                self.advance()?;
                true
            } else {
                false
            }
        } else {
            false
        };

        // read whole part while digit
        let whole = self.parse_whole_number(hex)?;
        
        // read fraction part while digit
        if self.current == b'.' || self.is_exp(hex) {
            let frac = if self.current == b'.' {
                self.advance()?;
                self.parse_frac_number(hex)?
            } else {
                0.
            };

            let exp = if self.is_exp(hex) {
                self.advance()?;
                let (signed, exp_sign) = self.sign_info();
                if signed { self.advance()? }
                self.parse_whole_number(hex)? as f64 * exp_sign as f64 
            } else {
                0f64
            };

            let base = if hex { 16f64 } else { 10f64 };
            let result = ((frac + whole as f64) * base.powf(exp)) as f64 ;
            Ok(Token::Float(sign as f64 * result))
        } else  if signed {
            Ok(Token::Signed(whole as i64 * sign))
        } else {
            Ok(Token::Unsigned(whole))
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        if self.is_whitespace() {
            return self.consume_whitespace();
        }
        match self.current {
            b'$' => self.consume_id(),
            b'"' => self.consume_string(),
            b'a'..=b'z' => self.consume_keyword(),
            b'0'..=b'9' | b'+' | b'-' => self.consume_number(),
            b'(' => {
                self.advance()?;
                if self.current == b';' {
                    self.consume_block_comment()
                } else {
                    Ok(Token::Open)
                }
            }
            b')' => {
                self.advance()?;
                Ok(Token::Close)
            },
            b';' => self.consume_line_comment(),
            _ => self.consume_other()
        }
    }
}

impl <R: Read> Iterator for Tokenizer<R> {
    type Item=Result<Token>;

    fn next(&mut self) -> Option<Result<Token>> {
        if self.eof {
            return None;
        }
        let token = self.next_token();
        Some(token) 
         
    }
}
