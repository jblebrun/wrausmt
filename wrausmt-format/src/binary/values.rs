use {
    super::{
        error::{BinaryParseError, BinaryParseErrorKind, Result, WithContext},
        leb128::ReadLeb128,
        BinaryParser,
    },
    std::{io::Read, marker::Sized},
    wrausmt_runtime::syntax::{
        types::{NumType, RefType, ValueType},
        Index, IndexSpace, Resolved,
    },
};

macro_rules! read_exact_bytes {
    ( $r:expr, $size:expr, $expect:expr ) => {{
        let mut buf = [0u8; $size];
        $r.read_exact(&mut buf).ctx("reading")?;
        if buf != $expect {
            Err(BinaryParseError::new(BinaryParseErrorKind::Unexpected {
                got:    Box::new(buf),
                expect: Box::new($expect),
            }))
        } else {
            Ok(())
        }
    }};
}

/// A collection of read helpers used by the various section reader traits.
impl<R: Read> BinaryParser<R> {
    pub(in crate::binary) fn read_magic(&mut self) -> Result<()> {
        read_exact_bytes!(self, 4, [0x00, 0x61, 0x73, 0x6d]).ctx("wrong magic")
    }

    pub(in crate::binary) fn read_version(&mut self) -> Result<()> {
        read_exact_bytes!(self, 4, [0x01, 0x00, 0x00, 0x00]).ctx("unsupported version")
    }

    /// Read a single byte, returning an errror for EOF.
    pub(in crate::binary) fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).ctx("reading next byte")?;
        Ok(buf[0])
    }

    /// Read a single byte, returning an error if it doesn't match the value
    /// provided.
    pub(in crate::binary) fn read_specific_byte(&mut self, expect: u8) -> Result<()> {
        read_exact_bytes!(self, 1, [expect])
    }

    /// Read a "name" field.
    /// Names are encoded as a vec(byte).
    pub(in crate::binary) fn read_name(&mut self) -> Result<String> {
        let bs = self.read_bytes()?;
        String::from_utf8(bs.to_vec()).ctx("parsing name data")
    }

    pub(in crate::binary) fn read_bytes(&mut self) -> Result<Box<[u8]>> {
        let length = self.read_u32_leb_128().ctx("parsing length")?;
        let mut bs: Vec<u8> = vec![0; length as usize];
        self.read_exact(&mut bs).ctx("reading name data")?;
        Ok(bs.into_boxed_slice())
    }

    /// Read a boolean field.
    /// A boolean field should only contain a value of 1 (for true) or 0 (for
    /// false).
    pub(in crate::binary) fn read_bool(&mut self) -> Result<bool> {
        let bool_byte = self.read_byte().ctx("fetching bool")?;
        match bool_byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(BinaryParseErrorKind::InvalidBoolValue(bool_byte).into()),
        }
    }

    pub(in crate::binary) fn read_value_type(&mut self) -> Result<ValueType> {
        ValueType::interpret(self.read_byte().ctx("fetching value type")?)
    }

    pub(in crate::binary) fn read_ref_type(&mut self) -> Result<RefType> {
        RefType::interpret(self.read_byte().ctx("fetching ref type")?)
    }

    pub(in crate::binary) fn read_vec<T>(
        &mut self,
        f: impl Fn(u32, &mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        let item_count = self.read_u32_leb_128().ctx("parsing count")?;
        (0..item_count).map(|i| f(i, self)).collect()
    }

    pub(in crate::binary) fn read_index_use<IS: IndexSpace>(
        &mut self,
    ) -> Result<Index<Resolved, IS>> {
        Ok(Index::unnamed(self.read_u32_leb_128().ctx("leb128")?))
    }
}

trait Interpret<T> {
    fn interpret(t: T) -> Result<Self>
    where
        Self: Sized;
}

impl Interpret<u8> for ValueType {
    fn interpret(byte: u8) -> Result<ValueType> {
        match byte {
            0x7F => Ok(NumType::I32.into()),
            0x7E => Ok(NumType::I64.into()),
            0x7D => Ok(NumType::F32.into()),
            0x7C => Ok(NumType::F64.into()),
            0x70 => Ok(RefType::Func.into()),
            0x6F => Ok(RefType::Extern.into()),
            _ => Err(BinaryParseErrorKind::InvalidValueType(byte).into()),
        }
    }
}

impl Interpret<u8> for RefType {
    fn interpret(byte: u8) -> Result<RefType> {
        match byte {
            0x70 => Ok(RefType::Func),
            0x6F => Ok(RefType::Extern),
            _ => Err(BinaryParseErrorKind::InvalidRefType(byte).into()),
        }
    }
}
