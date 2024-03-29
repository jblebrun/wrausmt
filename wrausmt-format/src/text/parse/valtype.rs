use {
    super::{super::token::Token, error::Result, Parser},
    std::io::Read,
    wrausmt_runtime::syntax::types::{NumType, RefType, ValueType},
};

impl<R: Read> Parser<R> {
    pub fn expect_valtype(&mut self) -> Result<ValueType> {
        self.try_valtype()?
            .ok_or(self.unexpected_token("expected value type"))
    }

    pub fn try_valtype(&mut self) -> Result<Option<ValueType>> {
        let result = match &self.current.token {
            Token::Keyword(kw) => match kw.as_str() {
                "funcref" => Some(ValueType::Ref(RefType::Func)),
                "externref" => Some(ValueType::Ref(RefType::Extern)),
                "i32" => Some(ValueType::Num(NumType::I32)),
                "i64" => Some(ValueType::Num(NumType::I64)),
                "f32" => Some(ValueType::Num(NumType::F32)),
                "f64" => Some(ValueType::Num(NumType::F64)),
                _ => None,
            },
            _ => None,
        };
        if result.is_some() {
            self.advance()?;
        }
        Ok(result)
    }

    pub fn expect_reftype(&mut self) -> Result<RefType> {
        self.try_reftype()?
            .ok_or(self.unexpected_token("expected ref type"))
    }

    pub fn try_reftype(&mut self) -> Result<Option<RefType>> {
        let result = match &self.current.token {
            Token::Keyword(kw) => match kw.as_str() {
                "funcref" => Some(RefType::Func),
                "externref" => Some(RefType::Extern),
                _ => None,
            },
            _ => None,
        };
        if result.is_some() {
            self.advance()?;
        }
        Ok(result)
    }

    pub fn try_heaptype(&mut self) -> Result<Option<RefType>> {
        let result = match &self.current.token {
            Token::Keyword(kw) => match kw.as_str() {
                "func" => Some(RefType::Func),
                "extern" => Some(RefType::Extern),
                _ => None,
            },
            _ => None,
        };
        if result.is_some() {
            self.advance()?;
        }
        Ok(result)
    }

    pub fn expect_heaptype(&mut self) -> Result<RefType> {
        self.try_heaptype()?
            .ok_or(self.unexpected_token("expected heap type"))
    }
}
