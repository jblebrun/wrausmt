use {
    super::{error::Result, leb128::ReadLeb128, BinaryParser, ParserReader},
    crate::{
        binary::error::{BinaryParseErrorKind, ParseResult},
        pctx,
    },
    wrausmt_runtime::syntax::{
        types::{GlobalType, Limits, MemType, TableType, ValueType},
        FParam, FResult, FunctionType, Resolved, TypeField, TypeUse,
    },
};

impl<R: ParserReader> BinaryParser<R> {
    pub fn read_types_section(&mut self) -> Result<Vec<TypeField>> {
        pctx!(self, "read types section");
        self.read_vec(|_, s| {
            let binary_type = s.read_byte_as_i7_leb_128().result(s)?;
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
        Ok(TypeUse::ById(self.read_index_use()?))
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
}
