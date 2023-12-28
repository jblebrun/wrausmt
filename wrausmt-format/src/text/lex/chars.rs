pub trait CharChecks {
    fn is_whitespace(&self) -> bool;
    fn is_keyword_start(&self) -> bool;
    fn is_idchar(&self) -> bool;
    fn as_hex_digit(&self) -> Option<u8>;
}

impl CharChecks for u8 {
    fn is_idchar(&self) -> bool {
        matches!(self,
            b'0'..=b'9' | b'A'..=b'Z'  | b'a'..=b'z' | b'!' | b'#' |
            b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-' | b'/' |
            b':' | b'<' | b'=' | b'>'  | b'?' | b'@' | b'\\' |
            b'^' | b'_' | b'`' | b'|'  | b'~' | b'.'
        )
    }

    fn is_keyword_start(&self) -> bool {
        self.is_ascii_lowercase()
    }

    fn is_whitespace(&self) -> bool {
        matches!(self, b' ' | b'\t' | b'\n' | b'\r')
    }

    fn as_hex_digit(&self) -> Option<u8> {
        match self {
            b'0'..=b'9' => Some(self - b'0'),
            b'A'..=b'F' => Some(self - b'A' + 10),
            b'a'..=b'f' => Some(self - b'a' + 10),
            _ => None,
        }
    }
}
