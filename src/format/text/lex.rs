use std::io::Read;
use std::iter::Iterator;
use crate::error::{Error, Result, ResultFrom};

#[derive(Debug)]
struct Tokenizer<R> {
    inner: R,
    current: u8,
    eof: bool,
    line: u32,
    pos: u32,
}

impl <R : Read> Tokenizer<R> {
    fn new(r: R) -> Result<Tokenizer<R>> {
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
            match self.current as char {
                '(' => {
                    self.advance()?;
                    if self.current as char == ';' {
                        depth += 1;
                    }
                },
                ';' => {
                    self.advance()?;
                    if self.current as char == ')' {
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
        if &result[0..6] == "nan:0x" {
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

    fn digit_val(&self, digit_byte: u8) -> u8 {
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
                result += exp * (digit - b'0') as u64;
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
            let digit_val = self.digit_val(digit);
            result += (digit_val) as f64 / exp as f64;
            exp *= exp_mult;
        }
        Ok(result)
    }

    fn sign_info(&mut self) -> (bool, i64) {
        match self.current as char {
            '+'  => (true, 1),
            '-'  => (true, -1),
            _ => (false, 1)
        }
    }

    fn consume_number(&mut self) -> Result<Token> {
        // read the sign if present
        if self.current as char == '+' {
            self.advance()?;
        }

        let (signed, sign) = self.sign_info();

        if signed {
            self.advance()?;
        }

        // if 0 check for x
        let hex = if self.current as char == '0' {
            self.advance()?;
            if self.current as char == 'x' {
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
        if self.current as char == '.' || self.is_exp(hex) {
            let frac = if self.current as char == '.' {
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
        match self.current as char {
            '$' => self.consume_id(),
            '"' => self.consume_string(),
            'a'..='z' => self.consume_keyword(),
            '0'..='9' | '+' | '-' => self.consume_number(),
            '(' => {
                self.advance()?;
                if self.current as char == ';' {
                    self.consume_block_comment()
                } else {
                    Ok(Token::Open)
                }
            }
            ')' => {
                self.advance()?;
                Ok(Token::Close)
            },
            ';' => self.consume_line_comment(),
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


#[allow(dead_code)]
#[derive(Debug, PartialEq)]
enum Token {
    Whitespace,
    LineComment,
    BlockComment,
    Keyword(String),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    String(String),
    Id(String),
    Open,
    Close,
    Reserved(String),
    Inf,
    NaN,
    NaNx(u32),
}


#[cfg(test)]
mod test {
    use super::Token;
    use super::Tokenizer;
    use crate::error::{Result, ResultFrom};
    macro_rules! expect_tokens {
        ( $tk:expr, $($t:expr),* ) => {
            $(
                assert_eq!($tk.next().unwrap()?, $t);
            )*
        }
    }

    fn printout<T : AsRef<[u8]>>(to_parse: T) -> Result<()> {
        let mut tokenizer = Tokenizer::new(to_parse.as_ref())?;

        for token in tokenizer {
            println!("{:?}", token.unwrap());
        }

        Ok(())
    }

    #[test]
    fn simple_parse() -> Result<()> {
        printout("(foo) \"hello\" (; comment (; nested ;) more ;)\n(yay)")
    }

    #[test]
    fn bare_string() -> Result<()> {
        printout("\"this is a string\"")
    }

    #[test]
    fn bare_unsigned() -> Result<()> {
        printout("3452")?;
        printout("0")?;
        Ok(())
    }

    #[test]
    fn bare_signed() -> Result<()> {
        printout("+3452")?;
        printout("-3452")?;
        printout("+0")?;
        printout("-0")?;
        Ok(())
    }

    #[test]
    fn bare_float() -> Result<()> {
        printout("1.5")?;
        printout("-1.5")?;
        printout("1.")?;
        printout("-1.")?;
        printout("-5e10")?;
        printout("-5.6e10")?;
        printout("-2.5e2")?;
        printout("-2.5e-2")?;
        Ok(())
    }

    #[test]
    fn bare_unsigned_hex() -> Result<()> {
        printout("0x60")?;
        printout("0xFF")?;
        printout("-0xFF")?;
        printout("+0xFF")?;
        printout("+0xFF.8p2")?;
        printout("-0xFF.7p-2")?;
        Ok(())
    }

    #[test]
    fn nans() -> Result<()> {
        printout("nan")?;
        printout("inf")?;
        printout("nan:0x56")?;
        Ok(())
    }

    #[test]
    fn seps() -> Result<()> {
        printout("100_000")?;
        printout("1.500_000_1")?;
        Ok(())
    }

    #[test]
    fn longer_test() -> Result<()> {
        let mut f = std::fs::File::open("testdata/locals.wat").wrap("opening file")?;
        let mut tokenizer = Tokenizer::new(&mut f)?;

        println!("here we go");
        for token in tokenizer {
            match token {
                Ok(t) => println!("{:?}", t),
                Err(e) => {
                    println!("ERR {:?}", e);
                    return Err(e)
                }
            }
        }

        Ok(())
    }
}
