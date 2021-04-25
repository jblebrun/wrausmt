use std::io::Read;
use super::{Expr, Field, Parser, TypeUse};
use crate::{error::{Result, ResultFrom}, types::ValueType};

// local := (local id? <valtype>)
#[derive(Debug, PartialEq)]
pub struct Local {
    id: Option<String>,
    valtype: ValueType
}

// Function fields may define a new function, or they may be an inline import.
#[derive(Debug, PartialEq)]
pub enum FuncContents {
    Inline{locals: Vec<Local>, body: Expr},
    Import{modname: String, name: String}
}

impl Default for FuncContents {
    fn default() -> Self { 
        FuncContents::Inline{locals: vec![], body: Expr::default() } 
    }
}

// func := (func id? <typeuse> <local>* <instr>*)
// instr := sequence of instr, or folded expressions
//
// Abbreviations:
// func := (func id? (export  <name>)*  ...)
// func := (func id? (import <modname> <name>) <typeuse>)
#[derive(Debug, PartialEq)]
pub struct FuncField {
    id: Option<String>,
    exports: Vec<String>,
    typeuse: TypeUse,
    contents: FuncContents
}

impl<R: Read> Parser<R> {
    // func := (func id? (export <name>)* (import <modname> <name>) <typeuse>)
    pub fn parse_func_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("func")? {
            return Ok(None)
        }

        let id = self.try_id()?;

        let mut exports: Vec<String> = vec![];

        while let Ok(Some(export)) = self.try_inline_export() {
            exports.push(export);
        }

        let import = self.try_inline_import()?;

        let typeuse = self.parse_type_use()?;

        let contents = if let Some((modname, name)) = import {
            self.expect_close().wrap("unexpected content in inline func import")?;
            FuncContents::Import{modname, name}
        } else {
            let mut locals: Vec<Local> = vec![];
            while let Some(more_locals) = self.try_locals()? {
                locals.extend(more_locals);
            }
            self.consume_expression()?;
            FuncContents::Inline{locals, body:Expr{}}
        };
        
        Ok(Some(Field::Func(FuncField {
            id,
            exports,
            typeuse,
            contents
        })))
    }   

    fn try_locals(&mut self) -> Result<Option<Vec<Local>>> {
        if !self.at_expr_start("local")? {
            return Ok(None)
        }
        let id = self.try_id()?;

        // Id specified, only one local in this group.
        if id.is_some() {
            let valtype = self.expect_valtype()?;
            self.expect_close()?;
            return Ok(Some(vec![Local{id,valtype}]))
        }

        // No id, any number of locals in this group.
        let mut result: Vec<Local> = vec![];

        while let Ok(Some(valtype)) = self.try_valtype() {
            result.push(Local{id:None, valtype})
        }

        self.expect_close()?;

        Ok(Some(result))
    }
}
