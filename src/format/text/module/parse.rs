use super::syntax::{DataField, ElemField, ElemList, ExportDesc, ExportField, Expr, FParam, FResult, Field, FuncField, FunctionType, GlobalField, ImportDesc, ImportField, Index, Local, MemoryContents, MemoryField, ModeEntry, Module, StartField, TableContents, TableField, TypeField, TypeUse};
use crate::error::{Result, ResultFrom};
use crate::format::text::Parser;
use crate::types::{GlobalType, Limits, RefType, TableType, ValueType};
use crate::{err, types::MemType};
use std::io::Read;

// Implementation for module-specific parsing functions.
impl<R: Read> Parser<R> {
    /// Attempt to parse the current token stream as a WebAssembly module.
    /// On success, a vector of sections is returned. They can be organized into a
    /// module object.
    pub fn try_module(&mut self) -> Result<Option<Module>> {
        if !self.try_expr_start("module")? {
            return Ok(None);
        }
        self.try_module_rest()
    }

    /// This is split away as a convenience for spec test parsing, so that we can
    /// parse the module expression header, and then check for binary/quote modules
    /// first, before attempting a normal module parse.
    pub fn try_module_rest(&mut self) -> Result<Option<Module>> {
        println!("PARSING MODULE!");

        let id = self.try_id()?;

        // section*
        let fields = self.zero_or_more(Self::try_field)?;

        self.expect_close()?;

        Ok(Some(Module { id, fields }))
    }

    // Parser should be located at the token immediately following a '('
    fn try_field(&mut self) -> Result<Option<Field>> {
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

    pub fn try_type_field(&mut self) -> Result<Option<Field>> {
        if !self.try_expr_start("type")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        self.expect_expr_start("func")?;

        let functiontype = self.try_function_type()?;

        // Close (func
        self.expect_close().wrap("ending type")?;

        // Close (type
        self.expect_close().wrap("ending type")?;

        Ok(Some(Field::Type(TypeField {
            id,
            functiontype
        })))
    }

    pub fn try_function_type(&mut self) -> Result<FunctionType> {
        Ok(FunctionType {
            params: self.zero_or_more_groups(Self::try_parse_fparam)?,
            results: self.zero_or_more_groups(Self::try_parse_fresult)?
        })
    }

    // func := (func id? (export <name>)* (import <modname> <name>) <typeuse>)
    pub fn try_func_field(&mut self) -> Result<Option<Field>> {
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

        self.consume_expression()?;

        Ok(Some(Field::Func(FuncField {
            id, exports, typeuse, locals, body: Expr {},
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

    pub fn try_table_field(&mut self) -> Result<Option<Field>> {
        if !self.try_expr_start("table")? {
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

    pub fn try_memory_field(&mut self) -> Result<Option<Field>> {
        if !self.try_expr_start("memory")? {
            return Ok(None);
        }
        self.consume_expression()?;
        Ok(Some(Field::Memory(MemoryField {
            id: None,
            exports: vec![],
            contents: MemoryContents::Import("foo".into()),
        })))
    }

    // import := (import <modname> <name> <exportdesc>)
    // exportdesc := (func <id>? <typeuse>)
    //             | (table <id>? <tabletype>)
    //             | (memory <id?> <memtype>)
    //             | (global <id?> <globaltype>)
    pub fn try_import_field(&mut self) -> Result<Option<Field>> {
        if !self.try_expr_start("import")? {
            return Ok(None);
        }

        let modname = self.expect_string()?;
        let name = self.expect_string()?;

        let (id, desc) = self.expect_importdesc()?;

        self.expect_close().wrap("closing import field")?;

        Ok(Some(Field::Import(ImportField {
            id,
            modname,
            name,
            desc,
        })))
    }

    pub fn expect_importdesc(&mut self) -> Result<(Option<String>, ImportDesc)> {
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
            err!("no valid importdesc found")
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
            self.expect_close().wrap("closing mut")?;
        }

        Ok(GlobalType { mutable, valtype })
    }

    pub fn try_export_field(&mut self) -> Result<Option<Field>> {
        if !self.try_expr_start("export")? {
            return Ok(None);
        }

        self.consume_expression()?;
        println!("COMPLETED EXPORTFIELD");
        Ok(Some(Field::Export(ExportField {
            name: "name".into(),
            exportdesc: ExportDesc::Func(TypeUse::default()),
        })))
    }

    pub fn try_global_field(&mut self) -> Result<Option<Field>> {
        if !self.try_expr_start("global")? {
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

    pub fn try_start_field(&mut self) -> Result<Option<Field>> {
        if !self.try_expr_start("start")? {
            return Ok(None);
        }
        self.consume_expression()?;
        Ok(Some(Field::Start(StartField {
            idx: Index::Numeric(42),
        })))
    }

    pub fn try_elem_field(&mut self) -> Result<Option<Field>> {
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

    pub fn try_data_field(&mut self) -> Result<Option<Field>> {
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
    fn parse_type_use(&mut self) -> Result<TypeUse> {
        let typeidx = if self.try_expr_start("type")? {
            let idx = self.parse_index()?;
            self.expect_close()?;
            Some(idx)
        } else {
            None
        };

        let functiontype = self.try_function_type()?;

        Ok(TypeUse {
            typeidx,
            functiontype
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
