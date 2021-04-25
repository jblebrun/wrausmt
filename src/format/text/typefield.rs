use std::io::Read;
use super::{Field, FParam, FResult, Parser};
use crate::error::{Result, ResultFrom};
use crate::err;

// type := (type id? <functype>)
// functype := (func <param>* <result>*)
#[derive(Debug, PartialEq, Default)]
pub struct TypeField {
    pub id: Option<String>,
    pub params: Vec<FParam>,
    pub results: Vec<FResult>
}

#[macro_export]
macro_rules! typefield {
    ( $id:literal; [$( $pt:ident $($pid:literal)?),*] -> [$($rt:ident),*] ) => {
        typefield! { Some($id.to_string()); [$($pt $($pid)?)*] -> [$($rt)*] }
    };
    ( [$($pt:ident $($pid:literal)?),*] -> [$($rt:ident),*] ) => {
        typefield! { None; [$($pt $($pid)?)*] -> [$($rt)*] }
    };
    ( $id:expr; [$($pt:ident $($pid:literal)?),*] -> [$($rt:ident),*] ) => {
        TypeField {
            id: $id,
            params: vec![$(wrausmt::fparam! { $($pid;)? $pt })*],
            results: vec![$(wrausmt::fresult! { $rt })*],
        }
    };
}

impl<R: Read> Parser<R> {
    pub fn parse_type_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("type")? {
            return Ok(None)
        }

        let id = self.try_id()?;
        
        let mut result = TypeField {
            id,
            params: vec![],
            results: vec![],
        };

        if !self.at_expr_start("func")? {
            return err!("Unexpected stuff in type")
        }

        while let Some(fparam) = self.try_parse_fparam().wrap("parsing params")? {
            result.params.push(fparam);
        }
        
        while let Some(fresult) = self.try_parse_fresult().wrap("parsing results")? {
            result.results.push(fresult);
        }

        // Close (func
        self.expect_close().wrap("ending type")?;

        // Close (type
        self.expect_close().wrap("ending type")?;

        Ok(Some(Field::Type(result)))
    }
}
