/// Convert the function name into a type-friendly name by converting all punctuation to '_'.
pub fn typename(s: &str) -> String {
    s.as_bytes()
        .iter()
        .map(|c| match *c as char {
            '.' | '_' => '_',
            _ => *c as char,
        })
        .collect()
}

/// Convert the operand format in the instruction table to the
pub fn operands(field: &str) -> String {
    let trimmed = field.trim_matches(&['(', ')'][..]);
    match trimmed {
        "" => "Operands::None".to_owned(),
        operand => format!("Operands::{}", operand),
    }
}

/// Quick-and-dirty hex parser. Doesn't do much validation, panics on failure.
pub fn hex(s: &str) -> u32 {
    let mut result = 0u32;
    let mut exp = 1u32;
    for digit in s.as_bytes().iter().rev() {
        match digit {
            b'0'..=b'9' => result += (digit - b'0') as u32 * exp,
            b'a'..=b'f' => result += (10 + digit - b'a') as u32 * exp,
            b'A'..=b'F' => result += (10 + digit - b'A') as u32 * exp,
            b'x' => break,
            _ => panic!("unhandling hex digit {}", digit),
        }
        exp *= 16;
    }
    result
}
