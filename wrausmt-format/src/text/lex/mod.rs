use {
    self::error::LexError,
    super::{
        num,
        token::{FileToken, Token},
    },
    chars::CharChecks,
    error::Result,
    std::{io::Read, iter::Iterator},
    wrausmt_common::true_or::TrueOr,
    wrausmt_runtime::syntax::{location::Location, Id},
};
mod chars;
pub mod error;

#[cfg(test)]
mod test;

/// A streaming WebAssembly tokenizer. It acts as an [Iterator] of
/// [Tokens][Token], parsing the [Read] source gradually as tokens are
/// requested.
#[derive(Debug)]
pub struct Tokenizer<R> {
    inner:    R,
    current:  u8,
    next:     u8,
    eof:      bool,
    finished: bool,
    location: Location,
}
impl<R: Read> Tokenizer<R> {
    pub fn new(r: R) -> Tokenizer<R> {
        Tokenizer {
            inner:    r,
            current:  0,
            next:     0,
            eof:      false,
            finished: false,
            location: Location::default(),
        }
    }

    // Read the next token bytes into a buffer up to the next separator.
    fn next_token(&mut self) -> Result<Token> {
        if self.location.line == 0 {
            self.location.nextline();
            self.advance()?;
            self.advance()?;
        }

        self.consume_ignorable()?;

        if self.finished {
            return Ok(Token::Eof);
        }

        match self.current {
            b'(' => self.consume_open(),
            b')' => self.consume_close(),
            b'"' => {
                let bytes = self.consume_string()?;
                if self.current.is_token_separator() {
                    Ok(Token::String(bytes[1..bytes.len() - 1].into()))
                } else {
                    self.consume_reserved(bytes)
                }
            }
            _ => {
                let bytes = self.consume_idchars()?;
                if self.finished || self.current.is_token_separator() {
                    self.interpret_idchars(bytes)
                } else {
                    self.consume_reserved(bytes)
                }
            }
        }
    }

    fn interpret_idchars(&mut self, bytes: Vec<u8>) -> Result<Token> {
        if let Ok(id) = TryInto::<Id>::try_into(bytes.as_slice()) {
            if id.as_str().starts_with('$') {
                return Ok(Token::Id(id));
            } else if let Some(n) = num::maybe_number(id.as_str()) {
                return Ok(Token::Number(n));
            } else if id.as_bytes()[0].is_keyword_start() {
                return Ok(Token::Keyword(id));
            }
        }
        self.consume_reserved(bytes)
    }

    /// Advance the current character to the next byte of the provided [Read].
    /// Updates line and position as well.
    /// If eof is reached, a flag is set. If [advance] is called after `eof` has
    /// been reached, panic occurs; this should not be a reachable state.
    fn advance(&mut self) -> Result<()> {
        let mut buf = [0u8; 1];
        let amount_read = self.inner.read(&mut buf)?;
        self.current = self.next;
        self.next = buf[0];
        if amount_read == 0 {
            if self.eof && self.finished {
                Err(LexError::UnexpectedEof)?;
            } else if self.eof && !self.finished {
                self.finished = true;
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

    /// Consume all contiguous whitespace and comments.
    fn consume_ignorable(&mut self) -> Result<()> {
        loop {
            match (self.current, self.next) {
                (w, _) if w.is_whitespace() => self.consume_whitespace()?,
                (b';', b';') => self.consume_line_comment()?,
                (b'(', b';') => self.consume_block_comment()?,
                _ => return Ok(()),
            }
        }
    }

    /// Consume the remaining chars up to the next separator as a
    /// [Token::Reserved], using the provided vector as the start of the token.
    fn consume_reserved(&mut self, bytes: Vec<u8>) -> Result<Token> {
        let mut bytes = bytes;
        while !self.current.is_token_separator() && !self.finished {
            match self.current {
                b'"' => bytes.extend(self.consume_string()?),
                _ => {
                    bytes.push(self.current);
                    self.advance()?
                }
            }
        }

        Ok(Token::Reserved(String::from_utf8_lossy(&bytes).to_string()))
    }

    /// Consume whitespace and return [Token::Whitespace]. Leaves the character
    /// pointer at the next non-whitespace token.
    fn consume_whitespace(&mut self) -> Result<()> {
        while self.current.is_whitespace() {
            self.advance()?
        }
        Ok(())
    }

    /// Consume a line comment and return [Token::LineComment]. Leaves the
    /// character position at the start of the next line.
    fn consume_line_comment(&mut self) -> Result<()> {
        (self.current == b';').true_or(LexError::UnexpectedChar(self.current as char))?;
        while !matches!(self.current, b'\n' | b'\r') {
            self.advance()?;
        }
        Ok(())
    }

    /// Consume a block comment, also handling nested comments, returning them
    /// all as one [Token::BlockComment].  Caller should have consumed '(',
    /// and we will be on the ';'. Leaves the character position one past
    /// the final ')'.
    fn consume_block_comment(&mut self) -> Result<()> {
        let mut depth = 1;
        // Past the (;
        self.advance()?;
        self.advance()?;
        while depth > 0 {
            match (self.current, self.next) {
                (b'(', b';') => {
                    depth += 1;
                    self.advance()?;
                    self.advance()?;
                }
                (b';', b')') => {
                    depth -= 1;
                    self.advance()?;
                    self.advance()?;
                }
                _ => self.advance()?,
            }
        }
        Ok(())
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

    fn consume_idchars(&mut self) -> Result<Vec<u8>> {
        let mut result: Vec<u8> = Vec::new();
        while self.current.is_idchar() {
            result.push(self.current);
            self.advance()?;
        }
        Ok(result)
    }

    /// Consume a string literal. Leaves the current character one position past
    /// the final '"'. The quotes are included in the result.
    fn consume_string(&mut self) -> Result<Vec<u8>> {
        let mut dest: Vec<u8> = Vec::new();
        dest.push(self.current);
        loop {
            self.advance()?;
            match self.current {
                b'\\' => {
                    self.advance()?;
                    let value = self.consume_escape()?;
                    dest.push(value);
                }
                b'"' => {
                    dest.push(self.current);
                    self.advance()?;
                    return Ok(dest);
                }
                _ => dest.push(self.current),
            }
        }
    }

    /// Handler for a ')', just returns [Token::Open] and advances the current
    /// character.
    fn consume_open(&mut self) -> Result<Token> {
        self.advance()?;
        Ok(Token::Open)
    }

    /// Handler for a ')', just returns [Token::Close] and advances the current
    /// character.
    fn consume_close(&mut self) -> Result<Token> {
        self.advance()?;
        Ok(Token::Close)
    }
}

impl<R: Read> Iterator for Tokenizer<R> {
    type Item = Result<FileToken>;

    fn next(&mut self) -> Option<Result<FileToken>> {
        match self.next_token() {
            Ok(Token::Eof) => None,
            t => Some(t.map(|token| FileToken {
                token,
                location: self.location,
            })),
        }
    }
}
