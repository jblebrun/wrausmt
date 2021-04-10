use super::leb128::ReadLeb128;
use crate::types::{RefType, ResultType, ValueType, NumType};
use crate::error::{ResultFrom, Error, Result};

macro_rules! read_exact_bytes {
    ( $r:expr, $size:expr, $expect:expr ) => {
        {
            let mut buf = [0u8; $size];
            $r.read_exact(&mut buf).wrap("reading")?;
            if buf != $expect {
                Err(Error::new(format!("mismatched bytes {:x?} -- expected {:x?}", buf, $expect)))
            } else {
                Ok(())
            }
        }
    }
}

/// A collection of read helpers used by the various section reader traits.
pub trait ReadWasmValues : ReadLeb128 {

    fn read_magic(&mut self) -> Result<()> {
        read_exact_bytes!(self, 4, [0x00, 0x61, 0x73, 0x6d]).wrap("wrong magic")
    }

    fn read_version(&mut self) -> Result<()> {
        read_exact_bytes!(self, 4, [0x01, 0x00, 0x00, 0x00]).wrap("unsupported version")
    }

    /// Read a single byte, returning an errror for EOF.
    fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).wrap("reading next byte")?;
        Ok(buf[0])
    }

    /// Read a single byte, returning an error if it doesn't match the value provided.
    fn read_specific_byte(&mut self, expect: u8) -> Result<()> {
        read_exact_bytes!(self, 1, [expect])
    }

    /// Read a "name" field.
    /// Names are encoded as a vec(byte).
    fn read_name(&mut self) -> Result<String> {
        let length = self.read_leb_128().wrap("parsing length")?;
        
        let mut bs: Vec<u8> = vec![0; length as usize];

        self.read_exact(&mut bs).wrap("reading name data")?;

        String::from_utf8(bs).wrap("parsing name data")
    }

    /// Read a boolean field.
    /// A boolean field should only contain a value of 1 (for true) or 0 (for false).
    fn read_bool(&mut self) -> Result<bool> {
       let bool_byte = self.read_byte().wrap("fetching bool")?;
       match bool_byte {
           0 => Ok(false),
           1 => Ok(true),
           _ => Err(Error::new(format!("invalid bool value {}", bool_byte)))
       }
    }

    fn read_value_type(&mut self) -> Result<ValueType> {
        convert_value_type(self.read_byte().wrap("fetching value type")?)
    }

    fn read_ref_type(&mut self) -> Result<RefType> {
        convert_ref_type(self.read_byte().wrap("fetching ref type")?)
    }
    
    fn read_result_type(&mut self) -> Result<Box<ResultType>> {
        let item_count = self.read_leb_128().wrap("parsing count")?;

        (0..item_count).map(|_| {
            self.read_value_type()
        }).collect()
    }

}

impl <I> ReadWasmValues for I where I:ReadLeb128 {} 

pub fn convert_value_type(byte: u8) -> Result<ValueType> {
    match byte {
        0x7F => Ok(ValueType::Num(NumType::I32)),
        0x7E => Ok(ValueType::Num(NumType::I64)),
        0x7D => Ok(ValueType::Num(NumType::F32)),
        0x7C => Ok(ValueType::Num(NumType::F64)),
        0x70 => Ok(ValueType::Ref(RefType::Func)),
        0x6F => Ok(ValueType::Ref(RefType::Extern)),
        _ => Err(Error::new(format!("{} does not encode a ValueType", byte)))
    }
}

pub fn convert_ref_type(byte: u8) -> Result<RefType> {
    match byte {
        0x70 => Ok(RefType::Func),
        0x6F => Ok(RefType::Extern),
        _ => Err(Error::new(format!("{} does not encode a RefType", byte)))
    }
}
