use {
    super::{
        error::{BinaryParseErrorKind, Result},
        leb128::ReadLeb128,
        BinaryParser,
    },
    crate::{binary::error::ParseResult, pctx},
    std::{io::Read, marker::Sized},
    wrausmt_runtime::syntax::{
        types::{NumType, RefType, ValueType},
        Index, IndexSpace, Resolved,
    },
};

macro_rules! read_exact_bytes {
    ( $r:expr, $size:expr, $expect:expr ) => {{
        let mut buf = [0u8; $size];
        $r.read_exact(&mut buf).result($r)?;
        if buf != $expect {
            Err($r.err(BinaryParseErrorKind::Unexpected {
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
        pctx!(self, "read magic");
        read_exact_bytes!(self, 4, [0x00, 0x61, 0x73, 0x6d])
    }

    pub(in crate::binary) fn read_version(&mut self) -> Result<()> {
        pctx!(self, "read version");
        read_exact_bytes!(self, 4, [0x01, 0x00, 0x00, 0x00])
    }

    /// Read a single byte, returning an errror for EOF.
    pub(in crate::binary) fn read_byte(&mut self) -> Result<u8> {
        pctx!(self, "read byte");
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).result(self)?;
        Ok(buf[0])
    }

    /// Read a single byte, returning an error if it doesn't match the value
    /// provided.
    pub(in crate::binary) fn read_specific_byte(&mut self, expect: u8) -> Result<()> {
        pctx!(self, "read specific byte");
        read_exact_bytes!(self, 1, [expect])
    }

    /// Read a "name" field.
    /// Names are encoded as a vec(byte).
    pub(in crate::binary) fn read_name(&mut self) -> Result<String> {
        pctx!(self, "read name");
        let bs = self.read_bytes()?;
        let r = String::from_utf8(bs.to_vec()).result(self)?;
        Ok(r)
    }

    pub(in crate::binary) fn read_bytes(&mut self) -> Result<Box<[u8]>> {
        pctx!(self, "read bytes");
        let length = self.read_u32_leb_128().result(self)?;
        let mut bs: Vec<u8> = vec![0; length as usize];
        self.read_exact(&mut bs).result(self)?;
        Ok(bs.into_boxed_slice())
    }

    /// Read a boolean field.
    /// A boolean field should only contain a value of 1 (for true) or 0 (for
    /// false).
    pub(in crate::binary) fn read_bool(&mut self) -> Result<bool> {
        pctx!(self, "read bool");
        let bool_byte = self.read_byte()?;
        match bool_byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(self.err(BinaryParseErrorKind::InvalidBoolValue(bool_byte))),
        }
    }

    pub(in crate::binary) fn read_value_type(&mut self) -> Result<ValueType> {
        pctx!(self, "read value type");
        let b = self.read_byte()?;
        ValueType::interpret(b).ok_or(self.err(BinaryParseErrorKind::InvalidValueType(b)))
    }

    pub(in crate::binary) fn read_ref_type(&mut self) -> Result<RefType> {
        pctx!(self, "read ref type");
        let b = self.read_byte()?;
        RefType::interpret(b).ok_or(self.err(BinaryParseErrorKind::InvalidRefType(b)))
    }

    pub(in crate::binary) fn read_vec<T>(
        &mut self,
        f: impl Fn(u32, &mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        pctx!(self, "read vec");
        let item_count = self.read_u32_leb_128().result(self)?;
        (0..item_count).map(|i| f(i, self)).collect()
    }

    pub(in crate::binary) fn read_index_use<IS: IndexSpace>(
        &mut self,
    ) -> Result<Index<Resolved, IS>> {
        pctx!(self, "read index use");
        Ok(Index::unnamed(self.read_u32_leb_128().result(self)?))
    }
}

trait Interpret<T> {
    fn interpret(t: T) -> Option<Self>
    where
        Self: Sized;
}

impl Interpret<u8> for ValueType {
    fn interpret(byte: u8) -> Option<ValueType> {
        match byte {
            0x7F => Some(NumType::I32.into()),
            0x7E => Some(NumType::I64.into()),
            0x7D => Some(NumType::F32.into()),
            0x7C => Some(NumType::F64.into()),
            0x70 => Some(RefType::Func.into()),
            0x6F => Some(RefType::Extern.into()),
            _ => None,
        }
    }
}

impl Interpret<u8> for RefType {
    fn interpret(byte: u8) -> Option<RefType> {
        match byte {
            0x70 => Some(RefType::Func),
            0x6F => Some(RefType::Extern),
            _ => None,
        }
    }
}
