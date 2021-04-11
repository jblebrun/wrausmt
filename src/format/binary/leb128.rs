use std::io::Read;
use crate::error::{Result, ResultFrom};
use crate::err;

// A macro to help with the very similar 32/64 bit implementations.
// This could also be done with type traits and using the type size
// To figure out the remainder and max byte, but this seemed more
// straight forward.
// $self - The self to call read_leb_128_byte on
// $ty - The type to return (u32 or u64)
// $maxbyte - The max number of bytes to expect - 1 (4 or 9)
// $remmask - The mask to use for checking remainder overflow (0xF0 or 0xFE)
// $signex - Whether or not to sign extend
macro_rules! leb_128 {
    ( $self:expr, $ty:ty, $maxbyte:expr, $remmask:expr, $signex:expr ) => {
        {
            let mut result: $ty = 0;
            let mut pos = 0;
            let maxbyte = $maxbyte;

            for idx in 0..=maxbyte {
                let (last, x) = $self.read_leb_128_byte()?;
                result |= (x as $ty) << pos;
                if last {
                    if idx == maxbyte {
                        let remainder = $remmask & x;
                        if remainder != 0 {
                            return err!("Too many bits to fit while LEB128 decoding");
                        }
                    }
                    if $signex && (x & 0x40 == 0x40) {
                        let signmask = <$ty>::MAX << pos;
                        result |= signmask;
                    }
                    return Ok(result);
                }
                pos += 7;
            }
            return err!("Did not reach final byte of LEB128")
        }
    }
}
pub trait ReadLeb128 : Read {

    fn read_leb_128_byte(&mut self) -> Result<(bool, u8)> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).wrap("reading next LEB byte")?;
        let completed = (buf[0] & 0x80) == 0x00;
        Ok((completed, buf[0] & 0x7f))
    }

    fn read_64_leb_128(&mut self, signex: bool) -> Result<u64> {
        leb_128!(self, u64, 9, 0xFE, signex)
    }

    fn read_32_leb_128(&mut self, signex: bool) -> Result<u32> {
        leb_128!(self, u32, 4, 0xF0, signex)
    }

    fn read_u32_leb_128(&mut self) -> Result<u32> { self.read_32_leb_128(false) }
    fn read_i32_leb_128(&mut self) -> Result<i32> { self.read_32_leb_128(true).map(|v| v as i32) }
    fn read_u64_leb_128(&mut self) -> Result<u64> { self.read_64_leb_128(false) }
    fn read_i64_leb_128(&mut self) -> Result<i64> { self.read_64_leb_128(true).map(|v| v as i64) }
}

impl <I:Read> ReadLeb128 for I {}


#[cfg(test)]
mod test {

    use super::ReadLeb128;
    use crate::assert_err_match;

    #[test]
    fn test_leb128_u32() {
        let data = vec![];
        let res = data.as_slice().read_u32_leb_128();
        assert_err_match!(res, "reading next LEB byte");

        let data: Vec<u8> = vec![8];
        let res = data.as_slice().read_u32_leb_128().unwrap();
        assert_eq!(res, 8);

        let data: Vec<u8> = vec![0x80, 0x01];
        let res = data.as_slice().read_u32_leb_128().unwrap();
        assert_eq!(res, 128);

        let data: Vec<u8> = vec![0x40];
        let res = data.as_slice().read_u32_leb_128().unwrap();
        assert_eq!(res, 64);

        let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F];
        let res = data.as_slice().read_u32_leb_128().unwrap();
        assert_eq!(res, 0xFFFFFFFF);
        
        let data: Vec<u8> = vec![0xF8, 0xFF, 0xFF, 0xFF, 0x0F];
        let res = data.as_slice().read_u32_leb_128().unwrap();
        assert_eq!(res, 0xFFFFFFF8);

        let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
        let res = data.as_slice().read_u32_leb_128();
        assert_err_match!(res, "Too many bits to fit");

        let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let res = data.as_slice().read_u32_leb_128();
        assert_err_match!(res, "Did not reach final");
    }

    #[test]
    fn test_leb128_u64() {
        let data = vec![];
        let res = data.as_slice().read_u64_leb_128();
        assert_err_match!(res, "reading next LEB byte");

        let data: Vec<u8> = vec![8];
        let res = data.as_slice().read_u64_leb_128().unwrap();
        assert_eq!(res, 8);

        let data: Vec<u8> = vec![0x80, 0x01];
        let res = data.as_slice().read_u64_leb_128().unwrap();
        assert_eq!(res, 128);

        let data: Vec<u8> = vec![0x40];
        let res = data.as_slice().read_u64_leb_128().unwrap();
        assert_eq!(res, 64);

        let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        let res = data.as_slice().read_u64_leb_128().unwrap();
        assert_eq!(res, 0xFFFFFFFFFFFFFFFF);

        let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
        let res = data.as_slice().read_u64_leb_128().unwrap();
        assert_eq!(res, 0x7FFFFFFFFFFFFFFF);

        let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
        let res = data.as_slice().read_u64_leb_128();
        assert_err_match!(res, "Too many bits");

        let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let res = data.as_slice().read_u64_leb_128();
        assert_err_match!(res, "Did not reach final byte");
    }

    #[test]
    fn test_leb128_i32() {
        let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F];
        let res = data.as_slice().read_i32_leb_128().unwrap();
        assert_eq!(res, -1);

        let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07];
        let res = data.as_slice().read_i32_leb_128().unwrap();
        assert_eq!(res, 0x7FFFFFFF);
        
        let data: Vec<u8> = vec![0x80, 0x41];
        let res = data.as_slice().read_i32_leb_128().unwrap();
        assert_eq!(res, -128);

        let data: Vec<u8> = vec![0x80, 0x80, 0x80, 0x80, 0x08];
        let res = data.as_slice().read_i32_leb_128().unwrap();
        assert_eq!(res, -0x80000000);
    }

    
    #[test]
    fn test_leb128_i64() {
        let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
        let res = data.as_slice().read_i64_leb_128().unwrap();
        assert_eq!(res, 0x7FFFFFFFFFFFFFFF);
        
        let data: Vec<u8> = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        let res = data.as_slice().read_i64_leb_128().unwrap();
        assert_eq!(res, -0x8000000000000000);

        let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        let res = data.as_slice().read_i64_leb_128().unwrap();
        assert_eq!(res, -1);
        
        let data: Vec<u8> = vec![0x80, 0x41];
        let res = data.as_slice().read_i64_leb_128().unwrap();
        assert_eq!(res, -128);

    }
}
