use {
    super::{
        error::{ParseErrorKind, Result},
        pctx, ParseResult, Parser,
    },
    crate::text::{
        module_builder::ModuleBuilder, parse_text_unresolved_instructions, token::Token,
    },
    std::io::Read,
    wrausmt_runtime::syntax::{
        types::{GlobalType, Limits, MemType, TableType},
        DataField, DataInit, ElemField, ExportDesc, ExportField, FParam, FResult, FuncField,
        FunctionType, GlobalField, Id, ImportDesc, ImportField, Index, IndexSpace, Local,
        MemoryField, MemoryIndex, ModeEntry, Module, Resolved, ResolvedState, StartField,
        TableField, TypeField, TypeUse, UncompiledExpr, Unresolved, Unvalidated, ValidatedState,
    },
};

#[derive(Debug, PartialEq)]
pub enum Field<R: ResolvedState, V: ValidatedState> {
    Type(TypeField),
    Func(FuncField<R, UncompiledExpr<R>>),
    Table(TableField, Option<ElemField<R, UncompiledExpr<R>>>),
    Memory(MemoryField, Option<Box<[u8]>>),
    Import(ImportField<R>),
    Export(ExportField<R, V>),
    Global(GlobalField<UncompiledExpr<R>>),
    Start(StartField<R, V>),
    Elem(ElemField<R, UncompiledExpr<R>>),
    Data(DataField<R, UncompiledExpr<R>>),
}

pub enum FParamId {
    Allowed,
    Forbidden,
}

const PAGE_SIZE: usize = 65536;

// Implementation for module-specific parsing functions.
impl<R: Read> Parser<R> {
    /// Attempt to parse the current token stream as a WebAssembly module.
    /// On success, a vector of sections is returned. They can be organized into
    /// a module object.
    pub fn try_module(
        &mut self,
    ) -> Result<Option<Module<Resolved, Unvalidated, UncompiledExpr<Resolved>>>> {
        pctx!(self, "try module");
        let (expect_close, id) = if self.try_expr_start("module")? {
            (true, self.try_id()?)
        } else {
            (false, None)
        };

        self.try_module_rest(id, expect_close)
    }

    pub fn parse_full_module(
        &mut self,
    ) -> Result<Module<Resolved, Unvalidated, UncompiledExpr<Resolved>>> {
        pctx!(self, "parse full module");
        self.assure_started()?;

        match self.try_module() {
            Ok(Some(m)) => {
                if self.current.token != Token::Eof {
                    return Err(self.err(ParseErrorKind::Incomplete));
                }
                Ok(m)
            }
            Ok(None) => Err(self.err(ParseErrorKind::Eof)),
            Err(e) => Err(e),
        }
    }

    fn fix_elem_table_id(ef: &mut ElemField<Unresolved, UncompiledExpr<Unresolved>>, idx: u32) {
        if let ModeEntry::Active(ref mut tp) = ef.mode {
            tp.tableuse.tableidx = Index::unnamed(idx)
        }
    }

    /// This is split away as a convenience for spec test parsing, so that we
    /// can parse the module expression header, and then check for
    /// binary/quote modules first, before attempting a normal module parse.
    pub fn try_module_rest(
        &mut self,
        id: Option<Id>,
        expect_close: bool,
    ) -> Result<Option<Module<Resolved, Unvalidated, UncompiledExpr<Resolved>>>> {
        pctx!(self, "try module rest");
        let mut module_builder = ModuleBuilder::new(id);

        // section*
        // As we parse each field, we populate the module.
        // This is a fairly involved match tree, since many fields may generate
        // multiple fields due to inline type defs or imports/exports.
        while let Some(field) = self.try_field().transpose() {
            let location = self.location();
            match field? {
                Field::Type(f) => module_builder.add_typefield(f).result(self)?,
                Field::Func(f) => module_builder.add_funcfield(f).result(self)?,
                Field::Table(t, e) => {
                    let tableidx = module_builder.tables();
                    module_builder.add_tablefield(t).result(self)?;
                    if let Some(mut e) = e {
                        Self::fix_elem_table_id(&mut e, tableidx);
                        module_builder.add_elemfield(e).result(self)?;
                    }
                }
                Field::Memory(f, d) => {
                    let memidx = module_builder.memories();
                    module_builder.add_memoryfield(f).result(self)?;
                    if let Some(d) = d {
                        module_builder
                            .add_datafield(DataField {
                                id: None,
                                data: d,
                                init: Some(DataInit {
                                    memidx: Index::unnamed(memidx),
                                    offset: parse_text_unresolved_instructions("i32.const 0"),
                                }),
                                location,
                            })
                            .result(self)?;
                    }
                }
                Field::Import(f) => module_builder.add_importfield(f).result(self)?,
                Field::Export(f) => module_builder.add_exportfield(f),
                Field::Global(f) => module_builder.add_globalfield(f).result(self)?,
                Field::Start(f) => module_builder.add_startfield(f).result(self)?,
                Field::Elem(f) => module_builder.add_elemfield(f).result(self)?,
                Field::Data(f) => module_builder.add_datafield(f).result(self)?,
            }
        }

        let result = if expect_close {
            self.expect_close()?;
            true
        } else {
            !module_builder.empty()
        };

        if result {
            Ok(Some(module_builder.build().result(self)?))
        } else {
            Ok(None)
        }
    }

    // Parser should be located at the token immediately following a '('
    fn try_field(&mut self) -> Result<Option<Field<Unresolved, Unvalidated>>> {
        pctx!(self, "try field");
        self.first_of(&[
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

    pub fn try_type_field(&mut self) -> Result<Option<Field<Unresolved, Unvalidated>>> {
        pctx!(self, "try type field");
        if !self.try_expr_start("type")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        self.expect_expr_start("func")?;

        let functiontype = self.try_function_type(FParamId::Allowed)?;

        // Close (func
        self.expect_close()?;

        // Close (type
        self.expect_close()?;

        Ok(Some(Field::Type(TypeField { id, functiontype })))
    }

    pub fn try_function_type(&mut self, fparam_id: FParamId) -> Result<FunctionType> {
        pctx!(self, "try function type");
        Ok(FunctionType {
            params:  self.zero_or_more_groups(match fparam_id {
                // Using closures makes the combinators a lot more complicated.
                // For this use case, it's simpler to just create variants for the
                // two types of FParam variants.
                FParamId::Allowed => Self::try_parse_fparam_id_allowed,
                FParamId::Forbidden => Self::try_parse_fparam_id_forbidden,
            })?,
            results: self.zero_or_more_groups(Self::try_parse_fresult)?,
        })
    }

    // func := (func id? (export <name>)* (import <modname> <name>) <typeuse>)
    pub fn try_func_field(&mut self) -> Result<Option<Field<Unresolved, Unvalidated>>> {
        pctx!(self, "try function field");
        let location = self.location();
        if !self.try_expr_start("func")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        let exports = self.zero_or_more(Self::try_inline_export)?;

        let import = self.try_inline_import()?;

        let typeuse = self.parse_type_use(FParamId::Allowed)?;

        if let Some((modname, name)) = import {
            self.expect_close()?;
            return Ok(Some(Field::Import(ImportField {
                modname,
                name,
                id,
                exports,
                desc: ImportDesc::Func(typeuse),
                location,
            })));
        }

        let locals = self.zero_or_more_groups(Self::try_locals)?;

        let instr = self.parse_instructions()?;
        self.expect_close()?;

        Ok(Some(Field::Func(FuncField {
            id,
            exports,
            typeuse,
            locals,
            body: UncompiledExpr { instr },
            location,
        })))
    }

    fn try_locals(&mut self) -> Result<Option<Vec<Local>>> {
        pctx!(self, "try locals");
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

    pub fn try_read_indices<IS: IndexSpace>(&mut self) -> Result<Vec<Index<Unresolved, IS>>> {
        pctx!(self, "try read indices");
        self.zero_or_more(Self::try_index)
    }

    fn try_inline_memory_data(&mut self) -> Result<Option<Box<[u8]>>> {
        pctx!(self, "try inline memory data");
        if !self.try_expr_start("data")? {
            return Ok(None);
        }

        // The data is written as a string, which may be split up into a possibly empty
        // sequence of individual string literals.
        let datas = self.zero_or_more(Self::try_wasm_string)?;

        let data = datas
            .into_iter()
            .flat_map::<Vec<u8>, _>(|d| d.into())
            .collect();

        self.expect_close()?;

        Ok(Some(data))
    }

    pub fn try_memory_field(&mut self) -> Result<Option<Field<Unresolved, Unvalidated>>> {
        pctx!(self, "try memory field");
        let location = self.location();
        if !self.try_expr_start("memory")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        let exports = self.zero_or_more(Self::try_inline_export)?;

        let import = self.try_inline_import()?;

        let inline_data = self.try_inline_memory_data()?;

        if let Some(inline_data) = inline_data {
            self.expect_close()?;
            let mut n = (inline_data.len() / PAGE_SIZE) as u32;
            if inline_data.len() % PAGE_SIZE > 0 {
                n += 1;
            }
            let memtype = MemType {
                limits: Limits {
                    lower: n,
                    upper: Some(n),
                },
            };
            return Ok(Some(Field::Memory(
                MemoryField {
                    id,
                    exports,
                    memtype,
                    location,
                },
                Some(inline_data),
            )));
        }

        let limits = self.expect_limits()?;

        let memtype = MemType { limits };

        if let Some(import) = import {
            self.expect_close()?;
            return Ok(Some(Field::Import(ImportField {
                id,
                modname: import.0,
                name: import.1,
                exports,
                desc: ImportDesc::Mem(memtype),
                location,
            })));
        }
        self.expect_close()?;
        Ok(Some(Field::Memory(
            MemoryField {
                id,
                exports,
                memtype,
                location,
            },
            None,
        )))
    }

    // import := (import <modname> <name> <exportdesc>)
    // exportdesc := (func <id>? <typeuse>)
    //             | (table <id>? <tabletype>)
    //             | (memory <id?> <memtype>)
    //             | (global <id?> <globaltype>)
    pub fn try_import_field(&mut self) -> Result<Option<Field<Unresolved, Unvalidated>>> {
        pctx!(self, "try import field");
        let location = self.location();
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
            exports: vec![],
            location,
        })))
    }

    pub fn expect_importdesc(&mut self) -> Result<(Option<Id>, ImportDesc<Unresolved>)> {
        pctx!(self, "expect importdesc");
        if self.try_expr_start("func")? {
            let id = self.try_id()?;
            let typeuse = self.parse_type_use(FParamId::Allowed)?;
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
            Err(self.unexpected_token("expected importdesc"))
        }
    }

    pub fn expect_tabletype(&mut self) -> Result<TableType> {
        pctx!(self, "expect tabletype");
        let limits = self.expect_limits()?;
        let reftype = self.expect_reftype()?;
        Ok(TableType { limits, reftype })
    }

    pub fn expect_globaltype(&mut self) -> Result<GlobalType> {
        pctx!(self, "expect globalype");
        let mutable = self.try_expr_start("mut")?;

        let valtype = self.expect_valtype()?;

        if mutable {
            self.expect_close()?;
        }

        Ok(GlobalType { mutable, valtype })
    }

    pub fn try_export_field(&mut self) -> Result<Option<Field<Unresolved, Unvalidated>>> {
        pctx!(self, "try export field");
        let location = self.location();
        if !self.try_expr_start("export")? {
            return Ok(None);
        }

        let name = self.expect_string()?;

        let exportdesc = self.expect_exportdesc()?;

        self.expect_close()?;

        Ok(Some(Field::Export(ExportField::new(
            name, exportdesc, location,
        ))))
    }

    fn expect_exportdesc(&mut self) -> Result<ExportDesc<Unresolved>> {
        pctx!(self, "expect export desc");
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
            Err(self.unexpected_token("expected exportdesc"))
        }
    }

    pub fn try_global_field(&mut self) -> Result<Option<Field<Unresolved, Unvalidated>>> {
        pctx!(self, "try global field");
        let location = self.location();
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
                exports,
                location,
            })));
        }

        let init = self.parse_instructions()?;

        self.expect_close()?;
        Ok(Some(Field::Global(GlobalField {
            id,
            exports,
            globaltype,
            init: UncompiledExpr { instr: init },
            location,
        })))
    }

    pub fn try_start_field(&mut self) -> Result<Option<Field<Unresolved, Unvalidated>>> {
        pctx!(self, "try start field");
        let location = self.location();
        if !self.try_expr_start("start")? {
            return Ok(None);
        }

        let idx = self.expect_index()?;

        self.expect_close()?;

        Ok(Some(Field::Start(StartField::new(idx, location))))
    }

    pub fn try_data_field(&mut self) -> Result<Option<Field<Unresolved, Unvalidated>>> {
        pctx!(self, "try data field");
        let location = self.location();
        if !self.try_expr_start("data")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        let memidx = self.try_memuse()?;

        let offset = self.try_offset_expression()?;

        // The data is written as a string, which may be split up into a possibly empty
        // sequence of individual string literals.
        let datas = self.zero_or_more(Self::try_wasm_string)?;

        let data = datas
            .into_iter()
            .flat_map::<Vec<u8>, _>(|d| d.into())
            .collect();

        self.expect_close()?;

        let init = offset.map(|offset| DataInit { memidx, offset });

        Ok(Some(Field::Data(DataField {
            id,
            data,
            init,
            location,
        })))
    }

    pub fn try_memuse(&mut self) -> Result<Index<Unresolved, MemoryIndex>> {
        pctx!(self, "try memuse");
        if !self.try_expr_start("memory")? {
            return Ok(Index::unnamed(0));
        }
        let memidx = self.expect_index()?;

        self.expect_close()?;
        Ok(memidx)
    }

    pub fn try_offset_expression(&mut self) -> Result<Option<UncompiledExpr<Unresolved>>> {
        self.try_item_or_offset_expression("offset")
    }

    pub fn try_item_expression(&mut self) -> Result<Option<UncompiledExpr<Unresolved>>> {
        self.try_item_or_offset_expression("item")
    }

    pub fn try_item_or_offset_expression(
        &mut self,
        which: &str,
    ) -> Result<Option<UncompiledExpr<Unresolved>>> {
        // (offset <instr>*)
        pctx!(self, "try offset expression");
        if self.try_expr_start(which)? {
            let instr = self.parse_instructions()?;
            self.expect_close()?;
            return Ok(Some(UncompiledExpr { instr }));
        }
        // The `(instr)` form used as a special shortcut form for `item` and `offset`.
        // It's expected that if we see an open paren, there should be a valid
        // instruction and then one close parent.
        Ok(self
            .try_folded_instruction()?
            .map(|instr| UncompiledExpr { instr }))
    }

    // Try to parse one "type use" section, in an import or function.
    // := (type <typeidx>)
    //  | (type <typeidx>) (param <id>? <type>)* (result <type>)*
    pub fn parse_type_use(&mut self, fparam_id: FParamId) -> Result<TypeUse<Unresolved>> {
        pctx!(self, "parse type use");
        let typeidx = if self.try_expr_start("type")? {
            let idx = self.expect_index()?;
            self.expect_close()?;
            Some(idx)
        } else {
            None
        };

        let functiontype = self.try_function_type(fparam_id)?;

        match (functiontype, typeidx) {
            (ft, None) => Ok(TypeUse::AnonymousInline(ft)),
            (ft, Some(ti)) if ft.is_void() => Ok(TypeUse::ByIndex(ti)),
            (functiontype, Some(index)) => Ok(TypeUse::NamedInline {
                functiontype,
                index,
            }),
        }
    }

    // Try to parse an inline export for a func, table, global, or memory.
    // := (export <name>)
    pub fn try_inline_export(&mut self) -> Result<Option<String>> {
        pctx!(self, "try inline export");
        if !self.try_expr_start("export")? {
            return Ok(None);
        }

        let data = self.expect_string()?;

        self.expect_close()?;

        Ok(Some(data))
    }

    // Try to parse an inline import for a func, table, global, or memory.
    // := (import <modname> <name>)
    pub fn try_inline_import(&mut self) -> Result<Option<(String, String)>> {
        pctx!(self, "try inline import");
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
    fn try_parse_fparam_id_allowed(&mut self) -> Result<Option<Vec<FParam>>> {
        self.try_parse_fparam(FParamId::Allowed)
    }

    fn try_parse_fparam_id_forbidden(&mut self) -> Result<Option<Vec<FParam>>> {
        self.try_parse_fparam(FParamId::Forbidden)
    }

    fn try_parse_fparam(&mut self, fparam_id: FParamId) -> Result<Option<Vec<FParam>>> {
        pctx!(self, "try parse fparam");
        if !self.try_expr_start("param")? {
            return Ok(None);
        }

        if matches!(fparam_id, FParamId::Allowed) {
            let id = self.try_id()?;
            if id.is_some() {
                let valuetype = self.expect_valtype()?;
                self.expect_close()?;
                return Ok(Some(vec![FParam { id, valuetype }]));
            }
        }

        // No id, any number of params in this group.
        let result = self.zero_or_more(Self::try_valtype_as_fparam)?;

        self.expect_close()?;

        Ok(Some(result))
    }

    fn try_valtype_as_fparam(&mut self) -> Result<Option<FParam>> {
        pctx!(self, "try valtype as fparam");
        Ok(self.try_valtype()?.map(|valuetype| FParam {
            id: None,
            valuetype,
        }))
    }

    fn try_valtype_as_fresult(&mut self) -> Result<Option<FResult>> {
        pctx!(self, "try valtype as fresult");
        Ok(self.try_valtype()?.map(|valuetype| FResult { valuetype }))
    }

    fn try_valtype_as_local(&mut self) -> Result<Option<Local>> {
        pctx!(self, "try valtype as local");
        Ok(self
            .try_valtype()?
            .map(|valtype| Local { id: None, valtype }))
    }

    // Try to parse a function result.
    // := (result <valtype>*)
    pub fn try_parse_fresult(&mut self) -> Result<Option<Vec<FResult>>> {
        pctx!(self, "try parse fresult");
        if !self.try_expr_start("result")? {
            return Ok(None);
        }

        let result = self.zero_or_more(Self::try_valtype_as_fresult)?;

        self.expect_close()?;

        Ok(Some(result))
    }

    // parse an index usage. It can be either a number or a named identifier.
    pub fn try_index<I: IndexSpace>(&mut self) -> Result<Option<Index<Unresolved, I>>> {
        pctx!(self, "try index");
        if let Some(id) = self.try_id()? {
            return Ok(Some(Index::named(id, 0)));
        }

        if let Some(val) = self.try_u32()? {
            return Ok(Some(Index::unnamed(val)));
        }

        Ok(None)
    }

    pub fn expect_index<I: IndexSpace>(&mut self) -> Result<Index<Unresolved, I>> {
        pctx!(self, "expect index");
        self.try_index()?.ok_or(self.unexpected_token("index"))
    }
}
