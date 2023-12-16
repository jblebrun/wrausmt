use std::io::Read;

#[derive(Debug)]
pub enum LEB128Error {
    IOError(std::io::Error),
    Overflow(Box<[u8]>),
    Unterminated(Box<[u8]>),
}

impl std::fmt::Display for LEB128Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOError(ioe) => write!(f, "LEB128Error::IOError: {}", ioe),
            Self::Overflow(bytes) => write!(f, "LEB128Error::Overflow for {:02x?}", bytes),
            Self::Unterminated(bytes) => write!(f, "LEB128Error::IOError: {:02x?}", bytes),
        }
    }
}

impl std::error::Error for LEB128Error {}

impl LEB128Error {
    pub fn overflow(bs: &[u8]) -> LEB128Error {
        LEB128Error::Overflow(bs.to_owned().into_boxed_slice())
    }
    pub fn unterminated(bs: &[u8]) -> LEB128Error {
        LEB128Error::Unterminated(bs.to_owned().into_boxed_slice())
    }
}

type Result<T> = std::result::Result<T, LEB128Error>;

// The final bit is the MSB. If it's unsigned, none of the high bits should be set.
// If it's signed, *all* of the high bits should be set.
fn validate_final_byte(result: &[u8], size: usize, signed: bool) -> Result<()> {
    let overflow_bit_count = 7 - (result.len() * 7) % size;
    let remainder_mask = 0xFF << overflow_bit_count;

    let signbit = result.last().unwrap() & 0x40 == 0x40;
    let expect = if signed && signbit {
        remainder_mask & 0x7f
    } else {
        0x00
    };

    let last = result.last().unwrap();
    if last & remainder_mask != expect {
        Err(LEB128Error::overflow(result))
    } else {
        Ok(())
    }
}

fn sign_extend(result: &mut Vec<u8>, size: usize, signed: bool) {
    if signed {
        let signbit = result.last().unwrap() & 0x40 == 0x40;
        if signbit {
            while result.len() < size {
                result.push(0xFF)
            }
        }
    }
}

fn read_leb_128_bytes(r: &mut impl Read, size: usize, signed: bool) -> Result<Vec<u8>> {
    let bytecount: usize = (size as f32 / 7.).ceil() as usize;
    let mut result = Vec::<u8>::with_capacity(bytecount);

    for br in r.bytes().take(bytecount) {
        let b = br.map_err(LEB128Error::IOError)?;
        result.push(b & 0x7f);
        if b & 0x80 == 0 {
            // Check for bit overflow for requested size
            if result.len() == bytecount {
                validate_final_byte(&result, size, signed)?;
            }

            sign_extend(&mut result, size, signed);

            return Ok(result);
        }
    }

    Err(LEB128Error::unterminated(&result))
}

// Generalized converter for both signed & unsigned LEB128 of any size.
// This does not handle size verification or signing, those are handled
// in read_leb_128_bytes.
fn parse_leb_128(buf: &[u8]) -> u64 {
    buf.iter().rev().fold(0, |acc, i| (acc << 7) | *i as u64)
}

pub trait ReadLeb128: Read + Sized {
    fn read_u32_leb_128(&mut self) -> Result<u32> {
        let bytes = read_leb_128_bytes(self, 32, false)?;
        Ok(parse_leb_128(&bytes) as u32)
    }
    fn read_i32_leb_128(&mut self) -> Result<i32> {
        let bytes = read_leb_128_bytes(self, 32, true)?;
        Ok(parse_leb_128(&bytes) as i32)
    }
    fn read_u64_leb_128(&mut self) -> Result<u64> {
        let bytes = read_leb_128_bytes(self, 64, false)?;
        Ok(parse_leb_128(&bytes) as u64)
    }
    fn read_i64_leb_128(&mut self) -> Result<i64> {
        let bytes = read_leb_128_bytes(self, 64, true)?;
        let uresult = parse_leb_128(&bytes);
        Ok(uresult as i64)
    }
}

impl<I: Read> ReadLeb128 for I {}

#[cfg(test)]
mod test {
    use super::parse_leb_128;

    #[test]
    fn test_parse_leb128() {
        let res = parse_leb_128(&[0x80u8, 0x01]);
        assert_eq!(res, 128);

        let res = parse_leb_128(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        assert_eq!(res, 0xFFFFFFFF);
    }
}
