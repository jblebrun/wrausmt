use std::io::Read;
use super::{Expr, Field, Parser, TypeUse};
use crate::{error::Result, types::ValueType};

#[derive(Debug)]
// local := (local id? <valtype>)
pub struct Local {
    id: Option<String>,
    vtype: ValueType
}

#[derive(Debug)]
#[allow(dead_code)]
// Function fields may define a new function, or they may be an inline import.
pub enum FuncContents {
    Inline{locals: Vec<Local>, body: Expr},
    Import(String)
}

impl Default for FuncContents {
    fn default() -> Self { 
        FuncContents::Inline{locals: vec![], body: Expr::default() } 
    }
}

#[derive(Debug, Default)]
// func := (func id? <typeuse> <local>* <instr>*)
// instr := sequence of instr, or folded expressions
//
// Abbreviations:
// func := (func id? (export  <name>)*  ...)
// func := (func id? (import <modname> <name>) <typeuse>)
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
        self.consume_expression()?; 
        Ok(Some(Field::Func(FuncField::default())))
    }   
}
