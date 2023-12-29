use {
    self::error::{ParseContext, ParseError, ParseErrorKind, Result},
    super::{
        lex::Tokenizer,
        string::WasmString,
        token::{FileToken, Token},
    },
    std::io::Read,
    wrausmt_runtime::syntax::Id,
};

mod combinator;
pub mod error;
mod instruction;
pub mod module;
mod num;
mod table;
mod valtype;

pub struct Parser<R: Read> {
    tokenizer:   Tokenizer<R>,
    pub current: FileToken,
    // 1 token of lookahead
    pub next:    FileToken,
}

trait Ignorable {
    fn ignorable(&self) -> bool;
}

impl Ignorable for Token {
    /// Returns true if the token is ignorable (whitespace, start, or comment)
    /// by the parser.
    fn ignorable(&self) -> bool {
        matches!(
            self,
            Token::Start | Token::Whitespace | Token::LineComment | Token::BlockComment
        )
    }
}

pub trait ParseResult<T> {
    fn result<R: Read>(self, parser: &Parser<R>) -> Result<T>;
}

impl<T, E: Into<ParseErrorKind>> ParseResult<T> for std::result::Result<T, E> {
    fn result<R: Read>(self, parser: &Parser<R>) -> Result<T> {
        self.map_err(|e| parser.err(e.into()))
    }
}

// Implementation for the basic token-handling methods.
impl<R: Read> Parser<R> {
    pub fn new_from_tokenizer(tokenizer: Tokenizer<R>) -> Parser<R> {
        Parser {
            tokenizer,
            current: FileToken::default(),
            next: FileToken::default(),
        }
    }

    pub fn new(reader: R) -> Parser<R> {
        Parser::new_from_tokenizer(Tokenizer::new(reader))
    }

    pub fn assure_started(&mut self) -> Result<()> {
        if self.current.token == Token::Start {
            self.advance()?;
            self.advance()?;
        }
        Ok(())
    }

    pub fn err(&self, err: ParseErrorKind) -> ParseError {
        ParseError::new(err, ParseContext {
            current: self.current.clone(),
            next:    self.next.clone(),
        })
    }

    pub fn unexpected_token(&self, name: impl Into<String>) -> ParseError {
        self.err(ParseErrorKind::UnexpectedToken(name.into()))
    }

    // Updates the lookahead token to the next value
    // provided by the tokenizer.
    fn next(&mut self) -> Result<()> {
        if self.next.token == Token::Eof {
            return Err(self.err(ParseErrorKind::Eof));
        }

        match self.tokenizer.next() {
            None => self.next.token = Token::Eof,
            Some(Ok(t)) => self.next = t,
            Some(Err(e)) => return Err(self.err(ParseErrorKind::LexError(e))),
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
        // println!(
        // "TOKENS ARE NOW {:?} {:?}",
        // self.current.token, self.next.token
        // );
        Ok(out)
    }

    pub fn state(&self) {
        println!("POSITION {:?} {:?}", self.current, self.next);
    }

    pub fn try_expr_start(&mut self, name: &str) -> Result<bool> {
        if self.current.token != Token::Open {
            return Ok(false);
        }
        match &self.next.token {
            Token::Keyword(k) if k.as_str() == name => {
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
            Token::Keyword(k) if k.as_str() == name => Ok(true),
            _ => Ok(false),
        }
    }

    fn expect_expr_start(&mut self, name: &str) -> Result<()> {
        if !self.try_expr_start(name)? {
            Err(self.unexpected_token("expression start"))
        } else {
            Ok(())
        }
    }

    pub fn expect_close(&mut self) -> Result<()> {
        match self.current.token {
            Token::Close => {
                self.advance()?;
                Ok(())
            }
            _ => Err(self.unexpected_token("expression close")),
        }
    }

    pub fn try_wasm_string(&mut self) -> Result<Option<WasmString>> {
        match self.current.token {
            Token::String(ref mut data) => {
                let data = std::mem::take(data);
                self.advance()?;
                Ok(Some(data))
            }
            _ => Ok(None),
        }
    }

    pub fn expect_wasm_string(&mut self) -> Result<WasmString> {
        self.try_wasm_string()?
            .ok_or(self.unexpected_token("wasm string literal"))
    }

    pub fn try_string(&mut self) -> Result<Option<String>> {
        let result = self.try_wasm_string()?;
        Ok(match result {
            Some(ws) => Some(ws.into_string().result(self)?),
            None => None,
        })
    }

    pub fn expect_string(&mut self) -> Result<String> {
        self.try_string()?
            .ok_or(self.unexpected_token("utf8string literal"))
    }

    pub fn try_id(&mut self) -> Result<Option<Id>> {
        match self.current.token {
            Token::Id(ref mut id) => {
                let id = std::mem::take(id);
                self.advance()?;
                Ok(Some(id))
            }
            _ => Ok(None),
        }
    }

    pub fn try_keyword(&mut self) -> Result<Option<Id>> {
        match self.current.token {
            Token::Keyword(ref mut id) => {
                let id = std::mem::take(id);
                self.advance()?;
                Ok(Some(id))
            }
            _ => Ok(None),
        }
    }

    pub fn peek_keyword(&self) -> Result<Option<&Id>> {
        match &self.current.token {
            Token::Keyword(id) => Ok(Some(id)),
            _ => Ok(None),
        }
    }

    pub fn take_keyword_if(&mut self, pred: impl Fn(&Id) -> bool) -> Result<Option<Id>> {
        match self.current.token {
            Token::Keyword(ref mut id) if pred(id) => {
                let id = std::mem::take(id);
                self.advance()?;
                Ok(Some(id))
            }
            _ => Ok(None),
        }
    }

    pub fn peek_next_keyword(&self) -> Result<Option<&Id>> {
        match &self.next.token {
            Token::Keyword(id) => Ok(Some(id)),
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
