use {
    super::{
        error::{Result, WithContext},
        leb128::ReadLeb128,
        BinaryParser,
    },
    std::io::Read,
    wrausmt_runtime::syntax::{
        types::{GlobalType, Limits, MemType, TableType, ValueType},
        FParam, FResult, FunctionType, Resolved, TypeField, TypeUse,
    },
};

impl<R: Read> BinaryParser<R> {
    pub fn read_types_section(&mut self) -> Result<Vec<TypeField>> {
        self.read_vec(|_, s| {
            s.read_specific_byte(0x60).ctx("checking type byte")?;
            Ok(TypeField {
                id:           None,
                functiontype: FunctionType {
                    params:  s.read_fparam().ctx("parsing params")?,
                    results: s.read_fresult().ctx("parsing result")?,
                },
            })
        })
    }

    fn read_result_type(&mut self) -> Result<Vec<ValueType>> {
        self.read_vec(|_, s| s.read_value_type())
    }

    fn read_fparam(&mut self) -> Result<Vec<FParam>> {
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
        Ok(self
            .read_result_type()?
            .into_iter()
            .map(|vt| FResult { valuetype: vt })
            .collect())
    }

    pub(in crate::binary) fn read_type_use(&mut self) -> Result<TypeUse<Resolved>> {
        Ok(TypeUse {
            typeidx:      Some(self.read_index_use()?),
            functiontype: FunctionType::default(),
        })
    }

    pub(in crate::binary) fn read_memory_type(&mut self) -> Result<MemType> {
        Ok(MemType {
            limits: self.read_limits().ctx("parsing limits")?,
        })
    }

    pub(in crate::binary) fn read_table_type(&mut self) -> Result<TableType> {
        Ok(TableType {
            reftype: self.read_ref_type().ctx("parsing reftype")?,
            limits:  self.read_limits().ctx("parsing limits")?,
        })
    }

    pub(in crate::binary) fn read_global_type(&mut self) -> Result<GlobalType> {
        Ok(GlobalType {
            valtype: self.read_value_type().ctx("parsing value")?,
            mutable: self.read_bool().ctx("parsing mutable")?,
        })
    }

    fn read_limits(&mut self) -> Result<Limits> {
        let has_upper = self.read_bool().ctx("parsing has upper")?;
        Ok(Limits {
            lower: self.read_u32_leb_128().ctx("parsing lower")?,
            upper: if has_upper {
                Some(self.read_u32_leb_128().ctx("parsing upper")?)
            } else {
                None
            },
        })
    }
}
