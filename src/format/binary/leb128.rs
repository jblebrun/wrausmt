use std::io::Read;
use crate::error::{Result, Error, ResultFrom};

pub trait ReadLeb128 : Read {
    fn read_leb_128(&mut self) -> Result<u32> {
        let mut result: u32 = 0;
        let mut pos = 0;
        let mut completed = false;

        for next in self.bytes() {
            let x = next.wrap("reading leb 128 byte")?;
            result |= ((x & 0x7f) as u32) << pos;
            if x & 0x80 == 0x00 { 
                completed = true;
                break;
            }
            pos += 7;
            if pos > 31 {
                return Err(Error::new("u32 LEB128 data is too long".to_string()));
            }
        }

        if !completed {
            return Err(Error::new("Reached end of input before parsing LEB128".to_string()));
        }
        
        Ok(result)
    }
}

impl <I:Read> ReadLeb128 for I {}

#[cfg(test)]
mod test {
    use super::ReadLeb128;

    #[test]
    fn test_leb128() {
        let data: Vec<u8> = vec![8];
        let res = data.as_slice().read_leb_128().unwrap();
        assert_eq!(res, 8);

        let data: Vec<u8> = vec![0x80, 0x01];
        let res = data.as_slice().read_leb_128().unwrap();
        assert_eq!(res, 128);

        let data: Vec<u8> = vec![0x40];
        let res = data.as_slice().read_leb_128().unwrap();
        assert_eq!(res, 64);
    }
}
