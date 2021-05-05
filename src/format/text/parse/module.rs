use super::error::{ParseError, ParseErrorContext, Result};
use super::Parser;
use crate::syntax::{
    DataField, ElemField, ElemList, ExportDesc, ExportField, Expr, FParam, FResult, FuncField,
    FunctionType, GlobalField, ImportDesc, ImportField, Index, IndexSpace, Local, MemoryField,
    ModeEntry, Module, Resolved, StartField, TableField, TypeField, TypeUse, Unresolved,
};
use crate::types::MemType;
use crate::types::{GlobalType, Limits, RefType, TableType};
use crate::{
    format::text::{module_builder::ModuleBuilder, token::Token},
    syntax::ResolvedState,
};
use std::{collections::HashMap, io::Read};

#[derive(Debug, PartialEq)]
pub enum Field<R: ResolvedState> {
    Type(TypeField),
    Func(FuncField<R>),
    Table(TableField, Option<ElemField<R>>),
    Memory(MemoryField),
    Import(ImportField<R>),
    Export(ExportField<R>),
    Global(GlobalField<R>),
    Start(StartField<R>),
    Elem(ElemField<R>),
    Data(DataField<R>),
}

// Implementation for module-specific parsing functions.
impl<R: Read> Parser<R> {
    /// Attempt to parse the current token stream as a WebAssembly module.
    /// On success, a vector of sections is returned. They can be organized into a
    /// module object.
    pub fn try_module(&mut self) -> Result<Option<Module<Resolved>>> {
        if !self.try_expr_start("module")? {
            return Ok(None);
        }
        self.try_module_rest()
    }

    pub fn parse_full_module(&mut self) -> Result<Module<Resolved>> {
        match self.try_module() {
            Ok(Some(m)) => {
                if self.current.token != Token::Eof {
                    return Err(self.with_context(ParseError::Incomplete));
                }
                Ok(m)
            }
            Ok(None) => Err(self.with_context(ParseError::Eof)),
            Err(e) => Err(self.with_context(e)),
        }
    }

    pub fn with_context(&self, err: ParseError) -> ParseError {
        ParseError::WithContext(
            ParseErrorContext {
                context: self.context.clone(),
                current: self.current.clone(),
                next: self.next.clone(),
            },
            Box::new(err),
        )
    }

    /// This is split away as a convenience for spec test parsing, so that we can
    /// parse the module expression header, and then check for binary/quote modules
    /// first, before attempting a normal module parse.
    pub fn try_module_rest(&mut self) -> Result<Option<Module<Resolved>>> {
        let id = self.try_id()?;

        let mut module_builder = ModuleBuilder::new(id);

        // section*
        // As we parse each field, we populate the module.
        // This is a fairly involved match tree, since many fields may generate
        // multiple fields due to inline type defs or imports/exports.
        for field in std::iter::from_fn(|| self.try_field().transpose()) {
            match field? {
                Field::Type(f) => module_builder.add_typefield(f),
                Field::Func(f) => module_builder.add_funcfield(f),
                Field::Table(t, e) => {
                    module_builder.add_tablefield(t);
                    if let Some(e) = e {
                        module_builder.add_elemfield(e);
                    }
                }
                Field::Memory(f) => module_builder.add_memoryfield(f),
                Field::Import(f) => module_builder.add_importfield(f),
                Field::Export(f) => module_builder.add_exportfield(f),
                Field::Global(f) => module_builder.add_globalfield(f),
                Field::Start(f) => module_builder.add_startfield(f),
                Field::Elem(f) => module_builder.add_elemfield(f),
                Field::Data(f) => module_builder.add_datafield(f),
            }
        }

        self.expect_close()?;

        Ok(Some(module_builder.build()?))
    }

    // Parser should be located at the token immediately following a '('
    fn try_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        self.first_of(&[
            Self::try_type_field,
            Self::try_type_field,
            Self::try_func_field,
            Self::try_table_field,
            Self::try_memory_field,
            Self::try_import_field,
            Self::try_export_field,
            Self::try_global_field,
            Self::try_start_field,
            Self::try_elem_field,
            Self::try_data_field,
        ])
    }

    pub fn try_type_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        if !self.try_expr_start("type")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        self.expect_expr_start("func")?;

        let functiontype = self.try_function_type()?;

        // Close (func
        self.expect_close()?;

        // Close (type
        self.expect_close()?;

        Ok(Some(Field::Type(TypeField { id, functiontype })))
    }

    pub fn try_function_type(&mut self) -> Result<FunctionType> {
        Ok(FunctionType {
            params: self.zero_or_more_groups(Self::try_parse_fparam)?,
            results: self.zero_or_more_groups(Self::try_parse_fresult)?,
        })
    }

    // func := (func id? (export <name>)* (import <modname> <name>) <typeuse>)
    pub fn try_func_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        if !self.try_expr_start("func")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        let exports = self.zero_or_more(Self::try_inline_export)?;

        let import = self.try_inline_import()?;

        let typeuse = self.parse_type_use()?;

        if let Some((modname, name)) = import {
            self.expect_close()?;
            return Ok(Some(Field::Import(ImportField {
                modname,
                name,
                id,
                desc: ImportDesc::Func(typeuse),
            })));
        }

        let locals = self.zero_or_more_groups(Self::try_locals)?;

        let instr = self.parse_instructions()?;
        self.expect_close()?;

        let mut idx = 0;
        let mut localindices = HashMap::default();
        for p in &typeuse.functiontype.params {
            if let Some(id) = &p.id {
                localindices.insert(id.to_owned(), idx);
            }
            idx += 1;
        }
        for l in &locals {
            if let Some(id) = &l.id {
                localindices.insert(id.to_owned(), idx);
            }
            idx += 1;
        }

        Ok(Some(Field::Func(FuncField {
            id,
            exports,
            typeuse,
            locals,
            body: Expr { instr },
            localindices,
        })))
    }

    fn try_locals(&mut self) -> Result<Option<Vec<Local>>> {
        if !self.try_expr_start("local")? {
            return Ok(None);
        }
        let id = self.try_id()?;

        // Id specified, only one local in this group.
        let result = if id.is_some() {
            let valtype = self.expect_valtype()?;
            vec![Local { id, valtype }]
        } else {
            // No id, any number of locals in this group.
            self.zero_or_more(Self::try_valtype_as_local)?
        };

        self.expect_close()?;
        Ok(Some(result))
    }

    pub fn try_table_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        if !self.try_expr_start("table")? {
            return Ok(None);
        }
        self.consume_expression()?;
        Ok(Some(Field::Table(
            TableField {
                id: None,
                exports: vec![],
                tabletype: TableType {
                    limits: Limits::default(),
                    reftype: RefType::Func,
                },
            },
            None,
        )))
    }

    pub fn try_memory_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        if !self.try_expr_start("memory")? {
            return Ok(None);
        }
        self.consume_expression()?;
        Ok(Some(Field::Memory(MemoryField {
            id: None,
            exports: vec![],
            memtype: MemType::default(),
            init: vec![],
        })))
    }

    // import := (import <modname> <name> <exportdesc>)
    // exportdesc := (func <id>? <typeuse>)
    //             | (table <id>? <tabletype>)
    //             | (memory <id?> <memtype>)
    //             | (global <id?> <globaltype>)
    pub fn try_import_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        if !self.try_expr_start("import")? {
            return Ok(None);
        }

        let modname = self.expect_string()?;
        let name = self.expect_string()?;

        let (id, desc) = self.expect_importdesc()?;

        self.expect_close()?;

        Ok(Some(Field::Import(ImportField {
            id,
            modname,
            name,
            desc,
        })))
    }

    pub fn expect_importdesc(&mut self) -> Result<(Option<String>, ImportDesc<Unresolved>)> {
        if self.try_expr_start("func")? {
            let id = self.try_id()?;
            let typeuse = self.parse_type_use()?;
            self.expect_close()?;
            Ok((id, ImportDesc::Func(typeuse)))
        } else if self.try_expr_start("table")? {
            let id = self.try_id()?;
            let tabletype = self.expect_tabletype()?;
            self.expect_close()?;
            Ok((id, ImportDesc::Table(tabletype)))
        } else if self.try_expr_start("memory")? {
            let id = self.try_id()?;
            let limits = self.expect_limits()?;
            self.expect_close()?;
            Ok((id, ImportDesc::Mem(MemType { limits })))
        } else if self.try_expr_start("global")? {
            let id = self.try_id()?;
            let globaltype = self.expect_globaltype()?;
            self.expect_close()?;
            Ok((id, ImportDesc::Global(globaltype)))
        } else {
            Err(ParseError::unexpected("importdesc"))
        }
    }

    pub fn expect_tabletype(&mut self) -> Result<TableType> {
        let limits = self.expect_limits()?;
        let reftype = self.expect_reftype()?;
        Ok(TableType { limits, reftype })
    }

    pub fn expect_globaltype(&mut self) -> Result<GlobalType> {
        let mutable = self.try_expr_start("mut")?;

        let valtype = self.expect_valtype()?;

        if mutable {
            self.expect_close()?;
        }

        Ok(GlobalType { mutable, valtype })
    }

    pub fn try_export_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        if !self.try_expr_start("export")? {
            return Ok(None);
        }

        let name = self.expect_string()?;

        let exportdesc = self.expect_exportdesc()?;

        self.expect_close()?;

        Ok(Some(Field::Export(ExportField { name, exportdesc })))
    }

    fn expect_exportdesc(&mut self) -> Result<ExportDesc<Unresolved>> {
        if self.try_expr_start("func")? {
            let index = self.expect_index()?;
            self.expect_close()?;
            Ok(ExportDesc::Func(index))
        } else if self.try_expr_start("table")? {
            let index = self.expect_index()?;
            self.expect_close()?;
            Ok(ExportDesc::Table(index))
        } else if self.try_expr_start("memory")? {
            let index = self.expect_index()?;
            self.expect_close()?;
            Ok(ExportDesc::Mem(index))
        } else if self.try_expr_start("global")? {
            let index = self.expect_index()?;
            self.expect_close()?;
            Ok(ExportDesc::Global(index))
        } else {
            Err(ParseError::unexpected("exportdesc"))
        }
    }

    pub fn try_global_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        if !self.try_expr_start("global")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        let exports = self.zero_or_more(Self::try_inline_export)?;

        let import = self.try_inline_import()?;

        let globaltype = self.expect_globaltype()?;

        if let Some(import) = import {
            self.expect_close()?;
            return Ok(Some(Field::Import(ImportField {
                id,
                modname: import.0,
                name: import.1,
                desc: ImportDesc::Global(globaltype),
            })));
        }

        let init = self.parse_instructions()?;

        self.expect_close()?;
        Ok(Some(Field::Global(GlobalField {
            id,
            exports,
            globaltype,
            init: Expr { instr: init },
        })))
    }

    pub fn try_start_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        if !self.try_expr_start("start")? {
            return Ok(None);
        }

        let idx = self.expect_index()?;

        self.expect_close()?;

        Ok(Some(Field::Start(StartField { idx })))
    }

    pub fn try_elem_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        if !self.try_expr_start("elem")? {
            return Ok(None);
        }
        self.consume_expression()?;
        Ok(Some(Field::Elem(ElemField {
            id: None,
            mode: ModeEntry::Passive,
            elemlist: ElemList {
                reftype: RefType::Func,
                items: vec![],
            },
        })))
    }

    pub fn try_data_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        if !self.try_expr_start("data")? {
            return Ok(None);
        }
        self.consume_expression()?;
        Ok(Some(Field::Data(DataField {
            id: None,
            data: vec![],
            init: None,
        })))
    }

    // Try to parse one "type use" section, in an import or function.
    // := (type <typeidx>)
    //  | (type <typeidx>) (param <id>? <type>)* (result <type>)*
    pub fn parse_type_use(&mut self) -> Result<TypeUse<Unresolved>> {
        let typeidx = if self.try_expr_start("type")? {
            let idx = self.expect_index()?;
            self.expect_close()?;
            Some(idx)
        } else {
            None
        };

        let functiontype = self.try_function_type()?;

        Ok(TypeUse {
            typeidx,
            functiontype,
        })
    }

    // Try to parse an inline export for a func, table, global, or memory.
    // := (export <name>)
    fn try_inline_export(&mut self) -> Result<Option<String>> {
        if !self.try_expr_start("export")? {
            return Ok(None);
        }

        let data = self.expect_string()?;

        self.expect_close()?;

        Ok(Some(data))
    }

    // Try to parse an inline import for a func, table, global, or memory.
    // := (import <modname> <name>)
    fn try_inline_import(&mut self) -> Result<Option<(String, String)>> {
        if !self.try_expr_start("import")? {
            return Ok(None);
        }

        let modname = self.expect_string()?;
        let name = self.expect_string()?;

        self.expect_close()?;

        Ok(Some((modname, name)))
    }

    // Try to parse a function parameter.
    // := (param $id <valtype>)
    //  | (param <valtype>*)
    fn try_parse_fparam(&mut self) -> Result<Option<Vec<FParam>>> {
        if !self.try_expr_start("param")? {
            return Ok(None);
        }

        let id = self.try_id()?;
        if id.is_some() {
            let valuetype = self.expect_valtype()?;
            self.expect_close()?;
            return Ok(Some(vec![FParam { id, valuetype }]));
        }

        // No id, any number of params in this group.
        let result = self.zero_or_more(Self::try_valtype_as_fparam)?;

        self.expect_close()?;

        Ok(Some(result))
    }

    fn try_valtype_as_fparam(&mut self) -> Result<Option<FParam>> {
        Ok(self.try_valtype()?.map(|valuetype| FParam {
            id: None,
            valuetype,
        }))
    }

    fn try_valtype_as_fresult(&mut self) -> Result<Option<FResult>> {
        Ok(self.try_valtype()?.map(|valuetype| FResult { valuetype }))
    }

    fn try_valtype_as_local(&mut self) -> Result<Option<Local>> {
        Ok(self
            .try_valtype()?
            .map(|valtype| Local { id: None, valtype }))
    }

    // Try to parse a function result.
    // := (result <valtype>*)
    fn try_parse_fresult(&mut self) -> Result<Option<Vec<FResult>>> {
        if !self.try_expr_start("result")? {
            return Ok(None);
        }

        let result = self.zero_or_more(Self::try_valtype_as_fresult)?;

        self.expect_close()?;

        Ok(Some(result))
    }

    // parse an index usage. It can be either a number or a named identifier.
    pub fn try_index<I: IndexSpace>(&mut self) -> Result<Option<Index<Unresolved, I>>> {
        if let Some(id) = self.try_id()? {
            return Ok(Some(Index::named(id, 0)));
        }

        if let Some(val) = self.try_unsigned()? {
            return Ok(Some(Index::unnamed(val as u32)));
        }

        Ok(None)
    }

    pub fn expect_index<I: IndexSpace>(&mut self) -> Result<Index<Unresolved, I>> {
        self.try_index()?
            .ok_or_else(|| ParseError::unexpected("index"))
    }
}
