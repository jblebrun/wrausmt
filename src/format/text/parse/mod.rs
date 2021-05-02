use super::lex::Tokenizer;
use std::io::Read;
use super::token::{FileToken, Token};
use error::{ParseError, Result};

pub mod error;
pub mod module;
mod combinator;
mod num;
mod valtype;
mod instruction;

pub struct Parser<R: Read> {
    tokenizer: Tokenizer<R>,
    pub current: FileToken,
    // 1 token of lookahead
    pub next: FileToken,
    context: Vec<String>
}

trait Ignorable {
    fn ignorable(&self) -> bool;
}

impl Ignorable for Token {
    /// Returns true if the token is ignorable (whitespace, start, or comment) by the parser.
    fn ignorable(&self) -> bool {
       matches!(self, Token::Start | Token::Whitespace | Token::LineComment | Token::BlockComment)
    }
}

// Implementation for the basic token-handling methods.
impl<R: Read> Parser<R> {
    pub fn new(tokenizer: Tokenizer<R>) -> Result<Parser<R>> {
        let mut p = Parser {
            tokenizer,
            current: FileToken::default(),
            next: FileToken::default(),
            context: vec![],
        };
        p.advance()?;
        p.advance()?;
        Ok(p)
    }

    // Updates the lookahead token to the next value
    // provided by the tokenizer.
    fn next(&mut self) -> Result<()> {
        if self.next.token == Token::Eof {
            return Err(ParseError::Eof)
        }

        match self.tokenizer.next() {
            None => self.next.token = Token::Eof,
            Some(Ok(t)) => self.next = t,
            Some(Err(e)) => return Err(ParseError::Tokenizer(e)),
        }
        Ok(())
    }

    // Advance to the next token, skipping all whitespace and comments.
    // Returns the current token to be owned by caller.
    pub fn advance(&mut self) -> Result<Token> {
        let out: Token = std::mem::take(&mut self.current.token);
        self.current = std::mem::take(&mut self.next);
        self.next()?;
        while self.next.token.ignorable() {
            self.next()?;
        }
        //println!("TOKEN IS NOW {:?}", self.current.token);
        Ok(out)
    }

    pub fn try_expr_start(&mut self, name: &str) -> Result<bool> {
        if self.current.token != Token::Open {
            return Ok(false);
        }
        match &self.next.token {
            Token::Keyword(k) if k == name => {
                self.advance()?;
                self.advance()?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn peek_expr_start(&mut self, name: &str) -> Result<bool> {
        if self.current.token != Token::Open {
            return Ok(false);
        }
        match &self.next.token {
            Token::Keyword(k) if k == name => {
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn expect_expr_start(&mut self, name: &str) -> Result<()> {
        if !self.try_expr_start(name)? {
            return Err(ParseError::unexpected("expression start"))
        }
        Ok(())
    }

    pub fn expect_close(&mut self) -> Result<()> {
        match self.current.token {
            Token::Close => {
                self.advance()?;
                Ok(())
            }
            _ => Err(ParseError::unexpected("expression close"))
        }
    }

    pub fn try_string(&mut self) -> Result<Option<String>> {
        match self.current.token {
            Token::String(ref mut data) => {
                let data = std::mem::take(data);
                self.advance()?;
                Ok(Some(data))
            }
            _ => Ok(None)
        }
    }

    pub fn expect_string(&mut self) -> Result<String> {
        self.try_string()?.ok_or_else(|| ParseError::unexpected("string literal"))
    }

    pub fn try_id(&mut self) -> Result<Option<String>> {
        match self.current.token {
            Token::Id(ref mut id) => {
                let id = std::mem::take(id);
                self.advance()?;
                Ok(Some(id))
            }
            _ => Ok(None),
        }
    }

    pub fn try_keyword(&mut self) -> Result<Option<String>> {
        match self.current.token {
            Token::Keyword(ref mut id) => {
                let id = std::mem::take(id);
                self.advance()?;
                Ok(Some(id))
            }
            _ => Ok(None),
        }
    }
    pub fn peek_keyword(&self) -> Result<Option<&str>> {
        match &self.current.token {
            Token::Keyword(id) => {
                Ok(Some(&id))
            }
            _ => Ok(None),
        }
    }

    pub fn take_keyword_if(&mut self, pred: fn(&str) -> bool) -> Result<Option<String>> {
        match self.current.token {
            Token::Keyword(ref mut id) if pred(id) => {
                let id = std::mem::take(id);
                self.advance()?;
                Ok(Some(id))
            },
            _ => Ok(None)
        }
    }

    pub fn peek_next_keyword(&self) -> Result<Option<&str>> {
        match &self.next.token {
            Token::Keyword(id) => {
                Ok(Some(&id))
            }
            _ => Ok(None),
        }
    }

    pub fn consume_expression(&mut self) -> Result<()> {
        let mut depth = 1;
        while depth > 0 {
            match self.current.token {
                Token::Open => depth += 1,
                Token::Close => depth -= 1,
                _ => (),
            };
            if depth == 0 {
                break;
            }
            self.advance()?;
        }
        self.advance()?;
        Ok(())
    }


}
