use std::string::FromUtf8Error;

/// A WebAssembly text format string.
/// They may contain any arbitrary bytes, not just valid UTF8.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct WasmString {
    bytes: Box<[u8]>,
}

impl From<WasmString> for Box<[u8]> {
    fn from(value: WasmString) -> Self {
        value.bytes
    }
}

impl From<WasmString> for Vec<u8> {
    fn from(value: WasmString) -> Self {
        value.bytes.into_vec()
    }
}

impl From<&[u8]> for WasmString {
    fn from(value: &[u8]) -> Self {
        WasmString {
            bytes: Box::from(value),
        }
    }
}

impl From<&str> for WasmString {
    fn from(value: &str) -> Self {
        value.as_bytes().into()
    }
}

impl TryFrom<WasmString> for String {
    type Error = FromUtf8Error;

    fn try_from(value: WasmString) -> Result<Self, Self::Error> {
        String::from_utf8(value.bytes.to_vec())
    }
}
