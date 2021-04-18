pub trait CharChecks { 
    fn is_whitespace(&self) -> bool;
    fn is_keyword_start(&self) -> bool;
    fn is_idchar(&self) -> bool;
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
        matches!(self, b'a'..=b'z')
    }

    fn is_whitespace(&self) -> bool {
        matches!(self, b' ' | b'\t' | b'\n' | b'\r')
    }
}
