use super::leb128::ReadLeb128;
use crate::{
    err,
    error::{Error, Result, ResultFrom},
    syntax::{FParam, FResult, FunctionType, Index, IndexSpace, Resolved, TypeUse},
    types::{GlobalType, Limits, MemType, NumType, RefType, TableType, ValueType},
};
use std::convert::TryFrom;

macro_rules! read_exact_bytes {
    ( $r:expr, $size:expr, $expect:expr ) => {{
        let mut buf = [0u8; $size];
        $r.read_exact(&mut buf).wrap("reading")?;
        if buf != $expect {
            Err(Error::new(format!(
                "mismatched bytes {:x?} -- expected {:x?}",
                buf, $expect
            )))
        } else {
            Ok(())
        }
    }};
}

/// A collection of read helpers used by the various section reader traits.
pub trait ReadWasmValues: ReadLeb128 + Sized {
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
        let bs = self.read_bytes()?;
        String::from_utf8(bs.to_vec()).wrap("parsing name data")
    }

    fn read_bytes(&mut self) -> Result<Box<[u8]>> {
        let length = self.read_u32_leb_128().wrap("parsing length")?;
        let mut bs: Vec<u8> = vec![0; length as usize];
        self.read_exact(&mut bs).wrap("reading name data")?;
        Ok(bs.into_boxed_slice())
    }

    /// Read a boolean field.
    /// A boolean field should only contain a value of 1 (for true) or 0 (for false).
    fn read_bool(&mut self) -> Result<bool> {
        let bool_byte = self.read_byte().wrap("fetching bool")?;
        match bool_byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => err!("invalid bool value {}", bool_byte),
        }
    }

    fn read_value_type(&mut self) -> Result<ValueType> {
        ValueType::try_from(self.read_byte().wrap("fetching value type")?)
    }

    fn read_ref_type(&mut self) -> Result<RefType> {
        RefType::try_from(self.read_byte().wrap("fetching ref type")?)
    }

    fn read_vec<T, F>(&mut self, f: F) -> Result<Vec<T>>
    where
        Self: Sized,
        F: Fn(u32, &mut Self) -> Result<T>,
    {
        let item_count = self.read_u32_leb_128().wrap("parsing count")?;
        (0..item_count).map(|i| f(i, self)).collect()
    }

    fn read_index_use<IS: IndexSpace>(&mut self) -> Result<Index<Resolved, IS>> {
        Ok(Index::unnamed(self.read_u32_leb_128().wrap("leb128")?))
    }

    fn read_type_use(&mut self) -> Result<TypeUse<Resolved>> {
        Ok(TypeUse {
            typeidx: Some(self.read_index_use()?),
            functiontype: FunctionType::default(),
        })
    }

    fn read_fparam(&mut self) -> Result<Vec<FParam>> {
        Ok(self
            .read_result_type()?
            .into_iter()
            .map(|vt| FParam {
                id: None,
                valuetype: vt,
            })
            .collect())
    }

    fn read_fresult(&mut self) -> Result<Vec<FResult>> {
        Ok(self
            .read_result_type()?
            .into_iter()
            .map(|vt| FResult { valuetype: vt })
            .collect())
    }

    fn read_result_type(&mut self) -> Result<Vec<ValueType>> {
        self.read_vec(|_, s| s.read_value_type())
    }

    fn read_memory_type(&mut self) -> Result<MemType> {
        Ok(MemType {
            limits: self.read_limits().wrap("parsing limits")?,
        })
    }

    fn read_table_type(&mut self) -> Result<TableType> {
        Ok(TableType {
            reftype: self.read_ref_type().wrap("parsing reftype")?,
            limits: self.read_limits().wrap("parsing limits")?,
        })
    }

    fn read_global_type(&mut self) -> Result<GlobalType> {
        Ok(GlobalType {
            valtype: self.read_value_type().wrap("parsing value")?,
            mutable: self.read_bool().wrap("parsing mutable")?,
        })
    }

    fn read_limits(&mut self) -> Result<Limits> {
        let has_upper = self.read_bool().wrap("parsing has upper")?;
        Ok(Limits {
            lower: self.read_u32_leb_128().wrap("parsing lower")?,
            upper: if has_upper {
                Some(self.read_u32_leb_128().wrap("parsing upper")?)
            } else {
                None
            },
        })
    }
}

impl<I> ReadWasmValues for I where I: ReadLeb128 {}

impl TryFrom<u8> for ValueType {
    type Error = Error;
    fn try_from(byte: u8) -> Result<ValueType> {
        match byte {
            0x7F => Ok(NumType::I32.into()),
            0x7E => Ok(NumType::I64.into()),
            0x7D => Ok(NumType::F32.into()),
            0x7C => Ok(NumType::F64.into()),
            0x70 => Ok(RefType::Func.into()),
            0x6F => Ok(RefType::Extern.into()),
            _ => err!("{:x?} is not a value type", byte),
        }
    }
}

impl TryFrom<u8> for RefType {
    type Error = Error;
    fn try_from(byte: u8) -> Result<RefType> {
        match byte {
            0x70 => Ok(RefType::Func),
            0x6F => Ok(RefType::Extern),
            _ => err!("{} does not encode a RefType", byte),
        }
    }
}
