use std::string::FromUtf8Error;

/// A WebAssembly text format string.
/// They may contain any arbitrary bytes, not just valid UTF8.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct WasmString {
    bytes: Box<[u8]>,
}

impl WasmString {
    pub fn from_bytes(bytes: Vec<u8>) -> WasmString {
        WasmString {
            bytes: bytes.into(),
        }
    }

    pub fn into_string(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.bytes.to_vec())
    }

    pub fn into_boxed_bytes(self) -> Box<[u8]> {
        self.bytes
    }
}
