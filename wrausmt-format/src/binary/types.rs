use {
    super::{error::Result, leb128::ReadLeb128, BinaryParser, ParserReader},
    crate::{
        binary::error::{BinaryParseErrorKind, ParseResult},
        pctx,
    },
    wrausmt_runtime::syntax::{
        types::{GlobalType, Limits, MemType, NumType, RefType, TableType, ValueType},
        FParam, FResult, FunctionType, Index, Resolved, TypeField, TypeUse,
    },
};

impl<R: ParserReader> BinaryParser<R> {
    pub fn read_types_section(&mut self) -> Result<Vec<TypeField>> {
        pctx!(self, "read types section");
        self.read_vec(|_, s| {
            let binary_type = s.read_i7_leb_128().result(s)?;
            match binary_type {
                -0x20 => Ok(TypeField {
                    id:           None,
                    functiontype: FunctionType {
                        params:  s.read_fparam()?,
                        results: s.read_fresult()?,
                    },
                }),
                bt => Err(s.err(BinaryParseErrorKind::InvalidFuncType(bt as u8))),
            }
        })
    }

    fn read_result_type(&mut self) -> Result<Vec<ValueType>> {
        pctx!(self, "read result type");
        self.read_vec(|_, s| s.read_value_type())
    }

    fn read_fparam(&mut self) -> Result<Vec<FParam>> {
        pctx!(self, "read fparam type");
        Ok(self
            .read_result_type()?
            .into_iter()
            .map(|vt| FParam {
                id:        None,
                valuetype: vt,
            })
            .collect())
    }

    fn read_fresult(&mut self) -> Result<Vec<FResult>> {
        pctx!(self, "read fresult type");
        Ok(self
            .read_result_type()?
            .into_iter()
            .map(|vt| FResult { valuetype: vt })
            .collect())
    }

    pub(in crate::binary) fn read_type_use(&mut self) -> Result<TypeUse<Resolved>> {
        pctx!(self, "read type use");
        Ok(TypeUse::ByIndex(self.read_index_use()?))
    }

    pub(in crate::binary) fn read_memory_type(&mut self) -> Result<MemType> {
        pctx!(self, "read memory type");
        Ok(MemType {
            limits: self.read_limits()?,
        })
    }

    pub(in crate::binary) fn read_table_type(&mut self) -> Result<TableType> {
        pctx!(self, "read table type");
        Ok(TableType {
            reftype: self.read_ref_type()?,
            limits:  self.read_limits()?,
        })
    }

    pub(in crate::binary) fn read_global_type(&mut self) -> Result<GlobalType> {
        pctx!(self, "read global type");
        Ok(GlobalType {
            valtype: self.read_value_type()?,
            mutable: self.read_bool()?,
        })
    }

    fn read_limits(&mut self) -> Result<Limits> {
        pctx!(self, "read limits");
        let has_upper = self.read_bool()?;
        Ok(Limits {
            lower: self.read_u32_leb_128().result(self)?,
            upper: if has_upper {
                Some(self.read_u32_leb_128().result(self)?)
            } else {
                None
            },
        })
    }

    pub(in crate::binary) fn read_blocktype(&mut self) -> Result<TypeUse<Resolved>> {
        pctx!(self, "read block type");
        let idx = self.read_i64_leb_128().result(self)?;
        Ok(match idx {
            -0x01 => TypeUse::single_result(NumType::I32.into()),
            -0x02 => TypeUse::single_result(NumType::I64.into()),
            -0x03 => TypeUse::single_result(NumType::F32.into()),
            -0x04 => TypeUse::single_result(NumType::F64.into()),
            -0x10 => TypeUse::single_result(RefType::Func.into()),
            -0x11 => TypeUse::single_result(RefType::Func.into()),
            -0x40 => TypeUse::default(),
            x if x > 0 && x <= u32::MAX as i64 => TypeUse::ByIndex(Index::unnamed(x as u32)),
            // TODO: This is not the right error.
            _ => Err(self.err(BinaryParseErrorKind::InvalidBlockType(idx)))?,
        })
    }
}
