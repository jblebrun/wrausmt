use self::error::LexError;

use super::token::{FileToken, Token};
use crate::format::{text::string::WasmString, Location};
use crate::syntax::Id;
mod chars;
pub mod error;
mod num;

use chars::CharChecks;
use error::{Result, WithContext};
use std::io::Read;
use std::iter::Iterator;

#[cfg(test)]
mod test;

/// A streaming WebAssembly tokenizer. It acts as an [Iterator] of [Tokens][Token],
/// parsing the [Read] source gradually as tokens are requested.
#[derive(Debug)]
pub struct Tokenizer<R> {
    inner: R,
    current: u8,
    eof: bool,
    location: Location,
}

fn keyword_or_reserved(idchars: Id) -> Token {
    if idchars.data()[0].is_keyword_start() {
        Token::Keyword(idchars)
    } else {
        Token::Reserved(idchars.as_str().into())
    }
}

impl<R: Read> Tokenizer<R> {
    pub fn new(r: R) -> Result<Tokenizer<R>> {
        let mut tokenizer = Tokenizer {
            inner: r,
            current: 0,
            eof: false,
            location: Location::default(),
        };
        tokenizer.location.nextline();
        tokenizer.advance()?;
        Ok(tokenizer)
    }

    fn next_token(&mut self) -> Result<Token> {
        if self.current.is_whitespace() {
            return self.consume_whitespace();
        }
        match self.current {
            b'"' => self.consume_string().ctx("while reading string literal"),
            b'(' => self.consume_open_or_block_comment(),
            b')' => self.consume_close(),
            b';' => self
                .consume_line_comment()
                .ctx("while consuming line comment"),
            b if b.is_idchar() => {
                let idchars = self.consume_idchars().ctx("while reading next token")?;
                if idchars.data()[0] == b'$' {
                    return Ok(Token::Id(idchars));
                }
                if let Some(n) = num::maybe_number(&idchars) {
                    return Ok(n);
                }
                Ok(keyword_or_reserved(idchars))
            }
            _ => Err(LexError::UnexpectedChar(self.current as char)),
        }
    }

    /// Advance the current character to the next byte of the provided [Read].
    /// Updates line and position as well.
    /// If eof is reached, a flag is set. If [advance] is called after `eof` has
    /// been reached, panic occurs; this should not be a reachable state.
    fn advance(&mut self) -> Result<()> {
        let mut buf = [0u8; 1];
        let amount_read = self.inner.read(&mut buf).ctx("reading")?;
        self.current = buf[0];
        if amount_read == 0 {
            if self.eof {
                panic!("unexpected eof");
            } else {
                self.eof = true;
            }
        } else if self.current == b'\n' {
            self.location.nextline();
        } else {
            self.location.nextchar();
        }
        Ok(())
    }

    /// Consume whitespace and return [Token::Whitespace]. Leaves the character pointer at the next
    /// non-whitespace token.
    fn consume_whitespace(&mut self) -> Result<Token> {
        while self.current.is_whitespace() {
            self.advance()?
        }
        Ok(Token::Whitespace)
    }

    /// Consume a line comment and return [Token::LineComment]. Leaves the character position at
    /// the start of the next line.
    fn consume_line_comment(&mut self) -> Result<Token> {
        if self.current != b';' {
            return Err(LexError::UnexpectedChar(self.current as char));
        }
        while !matches!(self.current, b'\n' | b'\r') {
            self.advance()?;
        }
        self.advance()?;
        Ok(Token::LineComment)
    }

    /// Consume a block comment, also handling nested comments, returning them all as one
    /// [Token::BlockComment].  Caller should have consumed '(', and we will be on the ';'.
    /// Leaves the character position one past the final ')'.
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
                }
                b';' => {
                    self.advance()?;
                    if self.current == b')' {
                        depth -= 1;
                        if depth == 0 {
                            self.advance()?;
                            break;
                        }
                    }
                }
                _ => self.advance()?,
            }
        }
        Ok(Token::BlockComment)
    }

    // Called during consume string to handle escape codes \xx
    fn consume_escape(&mut self) -> Result<u8> {
        if let Some(first) = self.current.as_hex_digit() {
            self.advance()?;
            return match self.current.as_hex_digit() {
                Some(second) => Ok(first << 4 | second),
                None => Err(LexError::InvalidEscape(format!(
                    "\\{:?}{:?}",
                    first, self.current
                ))),
            };
        }
        match self.current {
            b't' => Ok(b'\t'),
            b'n' => Ok(b'\n'),
            b'r' => Ok(b'\r'),
            b'"' => Ok(b'"'),
            b'\'' => Ok(b'\''),
            b'\\' => Ok(b'\\'),
            b'u' => Err(LexError::InvalidEscape(
                "Unicode escapes not supported yet".into(),
            )),
            _ => Err(LexError::InvalidEscape(format!("\\{}", self.current))),
        }
    }

    /// Consume a string literal. Leaves the current character one position past the final '"'.
    fn consume_string(&mut self) -> Result<Token> {
        let mut result: Vec<u8> = vec![];

        loop {
            self.advance()?;
            match self.current {
                b'\\' => {
                    self.advance()?;
                    let value = self.consume_escape()?;
                    result.push(value);
                }
                b'"' => {
                    self.advance()?;
                    let ws = WasmString::from_bytes(result);
                    return Ok(Token::String(ws));
                }
                _ => result.push(self.current),
            }
        }
    }

    /// Consume a contiguous block of idchars, which will eventually become either:
    /// A number, a keyword, an ID, or a reserved token.
    fn consume_idchars(&mut self) -> Result<Id> {
        let mut result: Vec<u8> = vec![];
        while self.current.is_idchar() {
            result.push(self.current);
            self.advance()?;
        }
        Ok(result.into())
    }

    /// Handler for a '(' - if followed by ';, consumes a block comment and returns
    /// [Token::BlockComment], otherwise just returns [Token::Open]. Leave the
    /// current character at the next character to parse.
    fn consume_open_or_block_comment(&mut self) -> Result<Token> {
        self.advance()?;
        if self.current == b';' {
            self.consume_block_comment()
                .ctx("while parsing block comment")
        } else {
            Ok(Token::Open)
        }
    }

    /// Handler for a ')', just returns [Token::Close] and advances the current character.
    fn consume_close(&mut self) -> Result<Token> {
        self.advance()?;
        Ok(Token::Close)
    }
}

impl<R: Read> Iterator for Tokenizer<R> {
    type Item = Result<FileToken>;

    fn next(&mut self) -> Option<Result<FileToken>> {
        if self.eof {
            return None;
        }
        let token = self.next_token().map(|t| self.location.token(t));
        Some(token)
    }
}
