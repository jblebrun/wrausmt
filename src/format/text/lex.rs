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
        } else {
            if self.current as char == '\n' {
                self.line += 1;
                self.pos = 0;
            } else {
                self.pos += 1;
            }
        }
        Ok(())
    }

    fn is_whitespace(&self) -> bool  {
        match self.current as char {
            ' ' | '\t' | '\n' | '\r' => true,
            _ => false
        }
    }

    fn is_idchar(&self) -> bool {
        match self.current as char {
            '0'..='9' | 'A'..='Z' | 'a'..='z' | '!' | '#' |
                '$' | '%' | '&' | '\'' | '*' | '+' | '/'  |
                ':' | '<' | '=' | '>' | '?' | '@' | '\\' | 
                '^' | '_' | '`' | '|' | '~' | '.'  => true,
                _ => false
        }
    }

    fn is_digit(&self, hex: bool) -> bool {
        match self.current as char {
            '_' | '0'..='9' => true,
            'a'..='f' | 'A'..='F' => hex,
            _ => false
        }
    }

    fn is_exp(&self, hex: bool) -> bool {
        match self.current as char {
            'e' | 'E' => !hex,
            'p' | 'P' => hex,
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
        while self.current as char != '\n' {
            self.advance()?;
        }
        self.advance()?;
        Ok(Token::LineComment)
    }

    // Caller will have consume (, and we will be on the ;
    fn consume_block_comment(&mut self) -> Result<Token> {
        println!("ENTER BC");
        let mut depth = 1;
        self.advance()?;
        while depth > 0 {
            println!("CHECK {}", self.current);
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
        let mut prev: char = '\0';

        loop {
            self.advance()?;
            if self.current as char == '"' && prev != '\\' {
                let as_string = String::from_utf8(result).wrap("bad utf8")?;
                self.advance()?;
                return Ok(Token::String(as_string))
            }
            result.push(self.current);
            prev = self.current as char;
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
        if !self.is_whitespace() && self.current as char != ')' {
            return Err(Error::new("Invalid char".to_string()));
        }
        let sresult = String::from_utf8(result).wrap("bad utf8")?;
        Ok(sresult)
    }

    fn consume_keyword(&mut self) -> Result<Token> {
        let result = self.consume_name()?;
        Ok(Token::Keyword(result))
    }

    fn consume_id(&mut self) -> Result<Token> {
        self.advance()?;
        let result = self.consume_name()?;
        Ok(Token::Id(result))
    }

    fn consume_digits(&mut self, hex: bool) -> Result<u64> {
        let mut digit_bytes: Vec<u8> = vec![];
        let whole = loop {
            if self.is_digit(hex) {
                digit_bytes.push(self.current);
                self.advance()?;
            } else {
                let mut exp = 1u64;
                let mut result = 0u64;
                for digit in digit_bytes.into_iter().rev() {
                    if digit as char != '_' {
                        result += exp * (digit - '0' as u8) as u64;
                        exp *= 10;
                    }
                }
                return Ok(result)
            }
        };
    }

    fn consume_number(&mut self) -> Result<Token> {
        // read the sign if present
        if self.current as char == '+' {
            self.advance()?;
        }

        let signed = match self.current as char {
            '+' | '-' => true,
            _ => false
        };

        let sign = match self.current as char {
            '-' => -1,
            _ => 1
        };

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
        let whole = self.consume_digits(hex)?;
        
        // read fraction part while digit
        if self.current as char == '.' || self.is_exp(hex) {
            let frac_raw = if self.current as char == '.' {
                self.advance()?;
                self.consume_digits(hex)?
            } else {
                0
            };

            let exp = if self.is_exp(hex) {
                self.advance()?;
                self.consume_digits(hex)?
            } else {
                1u64
            };

            println!("WHOLE {} FRAC {} EXP {}", whole, frac_raw, exp);
            let result = (whole * exp) as f64;
            Ok(Token::Float(sign as f64 * result))
        } else {
            if signed {
                Ok(Token::Signed(whole as i64 * sign))
            } else {
                Ok(Token::Unsigned(whole))
            }
        }
        // inf
        // nan
        // nan:0x
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
    Reserved(String)
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
    fn simple_test() -> Result<()> {
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
