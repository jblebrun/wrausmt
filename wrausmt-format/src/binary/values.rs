use {
    super::{
        error::{BinaryParseErrorKind, Result},
        leb128::ReadLeb128,
        BinaryParser, ParserReader,
    },
    crate::{binary::error::ParseResult, pctx},
    std::io::Read,
    wrausmt_common::true_or::TrueOr,
    wrausmt_runtime::syntax::{
        types::{NumType, RefType, ValueType},
        Index, IndexSpace, Resolved,
    },
};

impl<R: ParserReader> BinaryParser<R> {
    pub(in crate::binary) fn read_magic(&mut self) -> Result<()> {
        pctx!(self, "read magic");
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf).result(self)?;
        (&buf == b"\0asm").true_or_else(|| self.err(BinaryParseErrorKind::IncorrectMagic(buf)))
    }

    pub(in crate::binary) fn read_version(&mut self) -> Result<()> {
        pctx!(self, "read version");
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf).result(self)?;
        (&buf == b"\x01\0\0\0")
            .true_or_else(|| self.err(BinaryParseErrorKind::IncorrectVersion(buf)))
    }

    /// Read a single byte, returning an errror for EOF.
    pub(in crate::binary) fn read_byte(&mut self) -> Result<u8> {
        pctx!(self, "read byte");
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).result(self)?;
        Ok(buf[0])
    }

    /// Read a "name" field.
    /// Names are encoded as a vec(byte).
    pub(in crate::binary) fn read_name(&mut self) -> Result<String> {
        pctx!(self, "read name");
        let bs = self.read_bytes()?;
        String::from_utf8(bs.to_vec()).result(self)
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
        match self.read_byte_as_i7_leb_128().result(self)? {
            0 => Ok(false),
            1 => Ok(true),
            b => Err(self.err(BinaryParseErrorKind::InvalidBoolValue(b as u8))),
        }
    }

    pub(in crate::binary) fn read_value_type(&mut self) -> Result<ValueType> {
        pctx!(self, "read value type");
        match self.read_byte_as_i7_leb_128().result(self)? {
            -0x01 => Ok(NumType::I32.into()),
            -0x02 => Ok(NumType::I64.into()),
            -0x03 => Ok(NumType::F32.into()),
            -0x04 => Ok(NumType::F64.into()),
            -0x10 => Ok(RefType::Func.into()),
            -0x11 => Ok(RefType::Extern.into()),
            b => Err(self.err(BinaryParseErrorKind::InvalidValueType(b as u8))),
        }
    }

    pub(in crate::binary) fn read_ref_type(&mut self) -> Result<RefType> {
        pctx!(self, "read ref type");
        match self.read_byte_as_i7_leb_128().result(self)? {
            -0x10 => Ok(RefType::Func),
            -0x11 => Ok(RefType::Extern),
            b => Err(self.err(BinaryParseErrorKind::MalformedRefType(b as u8))),
        }
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
