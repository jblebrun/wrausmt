use super::lex::error::{LexError, Result as LexResult};
use std::string::FromUtf8Error;

/// A WebAssembly text format string.
/// They may contain any arbitrary bytes, not just valid UTF8.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct WasmString {
    bytes: Box<[u8]>,
}

fn parse_hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}
impl WasmString {
    pub fn from_bytes(bytes: &[u8]) -> LexResult<WasmString> {
        let mut result = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            match b {
                b'\\' => {
                    i += 1;
                    let next = bytes[i];
                    let c = if let Some(d1) = parse_hex_digit(next) {
                        i += 1;
                        let next = bytes[i];
                        if let Some(d2) = parse_hex_digit(next) {
                            (d1 << 4) | d2
                        } else {
                            return Err(LexError::InvalidEscape(next));
                        }
                    } else {
                        match next {
                            b't' => b'\t',
                            b'n' => b'\n',
                            b'r' => b'\r',
                            b'"' => b'\"',
                            b'\'' => b'\'',
                            b'\\' => b'\\',
                            b'u' => {
                                // TODO - unicode
                                b'0'
                            }
                            _ => return Err(LexError::InvalidEscape(next)),
                        }
                    };
                    result.push(c);
                }
                b => result.push(b),
            }
            i += 1;
        }
        Ok(WasmString {
            bytes: result.into_boxed_slice(),
        })
    }

    pub fn into_string(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.bytes.to_vec())
    }
}
