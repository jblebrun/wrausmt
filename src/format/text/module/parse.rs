use super::syntax::{
    DataField, ElemField, ElemList, ExportDesc, ExportField, Expr, FParam, FResult, Field,
    FuncContents, FuncField, GlobalField, ImportField, Index, Local, MemoryContents, MemoryField,
    ModeEntry, Module, StartField, TableContents, TableField, TypeField, TypeUse,
};
use crate::err;
use crate::error::{Result, ResultFrom};
use crate::format::text::{Parser, Token};
use crate::types::{GlobalType, Limits, RefType, TableType, ValueType};
use std::io::Read;

// Implementation for module-specific parsing functions.
impl<R: Read> Parser<R> {
    /// Attempt to parse the current token stream as a WebAssembly module.
    /// On success, a vector of sections is returned. They can be organized into a
    /// module object.
    pub fn parse_module(&mut self) -> Result<Module> {
        if self.current.token != Token::Open {
            return err!("Invalid start token {:?}", self.current);
        }
        self.advance()?;

        // Modules usually start with "(module". However, this is optional, and a module file can
        // be a list of top-levelo sections.
        if self.current.token.is_keyword("module") {
            self.advance()?;
        }

        // section*
        let mut result: Vec<Field> = vec![];
        while let Some(s) = self.parse_section()? {
            result.push(s);

            match self.current.token {
                Token::Open => (),
                Token::Close => break,
                _ => return err!("Invalid start token {:?}", self.current),
            }
        }

        Ok(Module {
            id: None,
            fields: result,
        })
    }

    // Parser should be located at the token immediately following a '('
    fn parse_section(&mut self) -> Result<Option<Field>> {
        if let Some(f) = self.parse_type_field()? {
            return Ok(Some(f));
        }
        if let Some(f) = self.parse_func_field()? {
            return Ok(Some(f));
        }
        if let Some(f) = self.parse_table_field()? {
            return Ok(Some(f));
        }
        if let Some(f) = self.parse_memory_field()? {
            return Ok(Some(f));
        }
        if let Some(f) = self.parse_import_field()? {
            return Ok(Some(f));
        }
        if let Some(f) = self.parse_export_field()? {
            return Ok(Some(f));
        }
        if let Some(f) = self.parse_global_field()? {
            return Ok(Some(f));
        }
        if let Some(f) = self.parse_start_field()? {
            return Ok(Some(f));
        }
        if let Some(f) = self.parse_elem_field()? {
            return Ok(Some(f));
        }
        if let Some(f) = self.parse_data_field()? {
            return Ok(Some(f));
        }
        return err!("no section found at {:?} {:?}", self.current, self.next);
    }

    pub fn parse_type_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("type")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        let mut result = TypeField {
            id,
            params: vec![],
            results: vec![],
        };

        if !self.at_expr_start("func")? {
            return err!("Unexpected stuff in type");
        }

        while let Some(fparams) = self.try_parse_fparam().wrap("parsing params")? {
            result.params.extend(fparams);
        }

        while let Some(fresults) = self.try_parse_fresult().wrap("parsing results")? {
            result.results.extend(fresults);
        }

        // Close (func
        self.expect_close().wrap("ending type")?;

        // Close (type
        self.expect_close().wrap("ending type")?;

        Ok(Some(Field::Type(result)))
    }

    // func := (func id? (export <name>)* (import <modname> <name>) <typeuse>)
    pub fn parse_func_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("func")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        let mut exports: Vec<String> = vec![];

        while let Ok(Some(export)) = self.try_inline_export() {
            exports.push(export);
        }

        let import = self.try_inline_import()?;

        let typeuse = self.parse_type_use()?;

        let contents = if let Some((modname, name)) = import {
            self.expect_close()
                .wrap("unexpected content in inline func import")?;
            FuncContents::Import { modname, name }
        } else {
            let mut locals: Vec<Local> = vec![];
            while let Some(more_locals) = self.try_locals()? {
                locals.extend(more_locals);
            }
            self.consume_expression()?;
            FuncContents::Inline {
                locals,
                body: Expr {},
            }
        };

        Ok(Some(Field::Func(FuncField {
            id,
            exports,
            typeuse,
            contents,
        })))
    }

    fn try_locals(&mut self) -> Result<Option<Vec<Local>>> {
        if !self.at_expr_start("local")? {
            return Ok(None);
        }
        let id = self.try_id()?;

        // Id specified, only one local in this group.
        if id.is_some() {
            let valtype = self.expect_valtype()?;
            self.expect_close()?;
            return Ok(Some(vec![Local { id, valtype }]));
        }

        // No id, any number of locals in this group.
        let mut result: Vec<Local> = vec![];

        while let Ok(Some(valtype)) = self.try_valtype() {
            result.push(Local { id: None, valtype })
        }

        self.expect_close()?;

        Ok(Some(result))
    }

    pub fn parse_table_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("table")? {
            return Ok(None);
        }
        self.consume_expression()?;
        Ok(Some(Field::Table(TableField {
            id: None,
            exports: vec![],
            tabletype: TableType {
                limits: Limits::default(),
                reftype: RefType::Func,
            },
            contents: TableContents::Inline { elems: None },
        })))
    }

    pub fn parse_memory_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("memory")? {
            return Ok(None);
        }
        self.consume_expression()?;
        Ok(Some(Field::Memory(MemoryField {
            id: None,
            exports: vec![],
            contents: MemoryContents::Import("foo".into()),
        })))
    }

    pub fn parse_import_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("import")? {
            return Ok(None);
        }
        self.consume_expression()?;
        Ok(Some(Field::Import(ImportField::default())))
    }

    pub fn parse_export_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("export")? {
            return Ok(None);
        }
        self.consume_expression()?;
        Ok(Some(Field::Export(ExportField {
            name: "name".into(),
            exportdesc: ExportDesc::Func(TypeUse::default()),
        })))
    }

    pub fn parse_global_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("global")? {
            return Ok(None);
        }

        self.consume_expression()?;
        Ok(Some(Field::Global(GlobalField {
            id: None,
            globaltype: GlobalType {
                mutable: false,
                valtype: ValueType::Ref(RefType::Func),
            },
            init: Expr {},
        })))
    }

    pub fn parse_start_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("start")? {
            return Ok(None);
        }
        self.consume_expression()?;
        Ok(Some(Field::Start(StartField {
            idx: Index::Numeric(42),
        })))
    }

    pub fn parse_elem_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("elem")? {
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

    pub fn parse_data_field(&mut self) -> Result<Option<Field>> {
        if !self.at_expr_start("data")? {
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
    fn parse_type_use(&mut self) -> Result<TypeUse> {
        if !self.at_expr_start("type")? {
            return err!("Expected type use");
        }

        let typeidx = self.parse_index()?;

        self.expect_close()?;

        let mut result = TypeUse {
            typeidx,
            params: vec![],
            results: vec![],
        };

        while let Some(fparams) = self.try_parse_fparam().wrap("parsing params")? {
            result.params.extend(fparams);
        }

        while let Some(fresults) = self.try_parse_fresult().wrap("parsing results")? {
            result.results.extend(fresults);
        }

        Ok(result)
    }

    // Try to parse an inline export for a func, table, global, or memory.
    // := (export <name>)
    fn try_inline_export(&mut self) -> Result<Option<String>> {
        if !self.at_expr_start("export")? {
            return Ok(None);
        }

        let data = self.expect_string()?;

        Ok(Some(data))
    }

    // Try to parse an inline import for a func, table, global, or memory.
    // := (import <modname> <name>)
    fn try_inline_import(&mut self) -> Result<Option<(String, String)>> {
        if !self.at_expr_start("import")? {
            return Ok(None);
        }

        let modname = self.expect_string()?;
        let name = self.expect_string()?;

        Ok(Some((modname, name)))
    }

    // Try to parse a function parameter.
    // := (param $id <valtype>)
    //  | (param <valtype>*)
    fn try_parse_fparam(&mut self) -> Result<Option<Vec<FParam>>> {
        if !self.at_expr_start("param")? {
            return Ok(None);
        }

        let id = self.try_id()?;
        if id.is_some() {
            let valuetype = self.expect_valtype()?;
            self.expect_close()?;
            return Ok(Some(vec![FParam { id, valuetype }]));
        }

        // No id, any number of params in this group.
        let mut result: Vec<FParam> = vec![];

        while let Ok(Some(valuetype)) = self.try_valtype() {
            result.push(FParam {
                id: None,
                valuetype,
            })
        }
        self.expect_close()?;

        Ok(Some(result))
    }

    // Try to parse a function result.
    // := (result <valtype>*)
    fn try_parse_fresult(&mut self) -> Result<Option<Vec<FResult>>> {
        if !self.at_expr_start("result")? {
            return Ok(None);
        }

        let mut result: Vec<FResult> = vec![];

        while let Ok(Some(valuetype)) = self.try_valtype() {
            result.push(FResult { valuetype })
        }

        self.expect_close()?;

        Ok(Some(result))
    }

    // parse an index usage. It can be either a number or a named identifier.
    fn parse_index(&mut self) -> Result<Index> {
        if let Some(id) = self.try_id()? {
            return Ok(Index::Named(id));
        }

        if let Some(val) = self.try_unsigned()? {
            return Ok(Index::Numeric(val as u32));
        }

        err!("No index found")
    }
}
