mod const_expression;
mod emitter;
mod validation;

pub use validation::{ValidationError, ValidationErrorKind};
use {
    self::{
        const_expression::compile_const_expr, emitter::ValidatingEmitter, validation::ModuleContext,
    },
    std::collections::HashSet,
    validation::{KindResult, Result},
    wrausmt_common::true_or::TrueOr,
    wrausmt_runtime::syntax::{
        location::Location,
        types::{MemType, NumType, TableType},
        CompiledExpr, DataField, DataInit, ElemField, ElemList, ExportDesc, ExportField, FuncField,
        FuncIndex, GlobalField, ImportDesc, ImportField, Index, MemoryField, ModeEntry, Module,
        Resolved, StartField, TableField, TablePosition, UncompiledExpr, Unvalidated, Validated,
    },
};

trait ToValidationError<T> {
    fn validation_error(self, location: Location) -> T;
}

impl<T> ToValidationError<Result<T>> for KindResult<T> {
    fn validation_error(self, location: Location) -> Result<T> {
        self.map_err(|kind| ValidationError::new(kind, location))
    }
}

// Compiles all functions in the module.
// It will consume the provided module, so you should clone the module if you
// need to do anything else with it later.
pub fn compile_module(
    module: Module<Resolved, Unvalidated, UncompiledExpr<Resolved>>,
) -> Result<Module<Resolved, Validated, CompiledExpr>> {
    // We need to create this now and hold onto it, beacuse the module will
    // change as we process its elements.
    let mut funcrefs: Vec<Index<Resolved, FuncIndex>> = Vec::new();
    let module_context = ModuleContext::new(&module)?;

    let mut module = module;

    let globals: Result<Vec<_>> = std::mem::take(&mut module.globals)
        .into_iter()
        .map(|g| compile_global(&module_context, &mut funcrefs, g))
        .collect();
    let globals = globals?;

    let elems: Result<Vec<_>> = std::mem::take(&mut module.elems)
        .into_iter()
        .enumerate()
        .map(|(i, e)| compile_elem(&module_context, &mut funcrefs, e, i))
        .collect();
    let elems = elems?;

    let data: Result<Vec<_>> = std::mem::take(&mut module.data)
        .into_iter()
        .enumerate()
        .map(|(i, d)| compile_data(&module_context, &mut funcrefs, d, i))
        .collect();
    let data = data?;

    let mut export_names: HashSet<String> = HashSet::new();
    let exports: Result<Vec<_>> = std::mem::take(&mut module.exports)
        .into_iter()
        .map(|e| {
            export_names
                .insert(e.name.clone())
                .true_or(ValidationErrorKind::DuplicateExport)
                .validation_error(e.location)?;
            validate_export(&module_context, &mut funcrefs, e)
        })
        .collect();
    let exports = exports?;

    let memories: Result<Vec<_>> = std::mem::take(&mut module.memories)
        .into_iter()
        .map(validate_memory)
        .collect();
    let memories = memories?;

    let tables: Result<Vec<_>> = std::mem::take(&mut module.tables)
        .into_iter()
        .map(validate_table)
        .collect();
    let tables = tables?;

    let imports: Result<Vec<_>> = std::mem::take(&mut module.imports)
        .into_iter()
        .map(validate_import)
        .collect();
    let imports = imports?;

    let module_context = module_context.update_func_refs(funcrefs);
    let funcs: Result<Vec<_>> = std::mem::take(&mut module.funcs)
        .into_iter()
        .map(|f| compile_func(&module_context, f))
        .collect();
    let funcs = funcs?;

    let start = validate_start(&module_context, module.start)?;

    Ok(Module {
        id: module.id,
        customs: module.customs,
        types: module.types,
        funcs,
        tables,
        memories,
        imports,
        exports,
        globals,
        start,
        elems,
        data,
    })
}

fn compile_func(
    module: &ModuleContext,
    func: FuncField<Resolved, UncompiledExpr<Resolved>>,
) -> Result<FuncField<Resolved, CompiledExpr>> {
    let body = ValidatingEmitter::function_body(module, &func)?;
    Ok(FuncField {
        id: func.id,
        exports: func.exports,
        typeuse: func.typeuse,
        locals: func.locals,
        body,
        location: func.location,
    })
}

fn compile_global(
    module: &ModuleContext,
    funcrefs: &mut Vec<Index<Resolved, FuncIndex>>,
    global: GlobalField<UncompiledExpr<Resolved>>,
) -> Result<GlobalField<CompiledExpr>> {
    let expect_type = global.globaltype.valtype;
    Ok(GlobalField {
        id:         global.id,
        exports:    global.exports,
        globaltype: global.globaltype,
        init:       compile_const_expr(&global.init, module, funcrefs, expect_type)
            .validation_error(global.location)?,
        location:   global.location,
    })
}

fn compile_elem(
    module: &ModuleContext,
    funcrefs: &mut Vec<Index<Resolved, FuncIndex>>,
    elem: ElemField<Resolved, UncompiledExpr<Resolved>>,
    ei: usize,
) -> Result<ElemField<Resolved, CompiledExpr>> {
    let elemlist = compile_elem_list(module, funcrefs, &elem.elemlist, &elem.location)?;
    Ok(ElemField {
        id: elem.id,
        mode: compile_elem_mode(
            module,
            funcrefs,
            elem.mode,
            elem.elemlist,
            ei,
            &elem.location,
        )?,
        elemlist,
        location: elem.location,
    })
}

fn compile_data(
    module: &ModuleContext,
    funcrefs: &mut Vec<Index<Resolved, FuncIndex>>,
    data: DataField<Resolved, UncompiledExpr<Resolved>>,
    di: usize,
) -> Result<DataField<Resolved, CompiledExpr>> {
    let dlen = data.data.len();
    Ok(DataField {
        id:       data.id,
        data:     data.data,
        init:     match data.init {
            Some(data_init) => Some(compile_data_init(
                module,
                funcrefs,
                data_init,
                dlen,
                di,
                &data.location,
            )?),
            None => None,
        },
        location: data.location,
    })
}

fn validate_export(
    module: &ModuleContext,
    funcrefs: &mut Vec<Index<Resolved, FuncIndex>>,
    export: ExportField<Resolved, Unvalidated>,
) -> Result<ExportField<Resolved, Validated>> {
    Ok(ExportField::new(
        export.name,
        compile_export_desc(module, funcrefs, export.exportdesc)
            .validation_error(export.location)?,
        export.location,
    ))
}

fn validate_start(
    module: &ModuleContext,
    start: Option<StartField<Resolved, Unvalidated>>,
) -> Result<Option<StartField<Resolved, Validated>>> {
    match start {
        Some(start) => {
            let f = (module.funcs.get(start.idx.value() as usize))
                .ok_or(ValidationErrorKind::UnknownFunc)
                .validation_error(start.location)?;
            (f.params.is_empty() && f.results.is_empty())
                .true_or(ValidationErrorKind::WrongStartFunctionType)
                .validation_error(start.location)?;
            Ok(Some(StartField::new(start.idx, start.location)))
        }
        None => Ok(None),
    }
}

fn validate_memory(memory: MemoryField<Unvalidated>) -> Result<MemoryField<Validated>> {
    Ok(MemoryField {
        id:       memory.id,
        exports:  memory.exports,
        memtype:  validate_memtype(memory.memtype).validation_error(memory.location)?,
        location: memory.location,
    })
}

fn validate_table(table: TableField<Unvalidated>) -> Result<TableField<Validated>> {
    Ok(TableField {
        id:        table.id,
        exports:   table.exports,
        tabletype: validate_tabletype(table.tabletype).validation_error(table.location)?,
        location:  table.location,
    })
}

fn validate_memtype(memtype: MemType<Unvalidated>) -> KindResult<MemType<Validated>> {
    if let Some(upper) = memtype.limits.upper {
        (memtype.limits.lower <= upper).true_or(ValidationErrorKind::InvalidLimits)?;
    }
    (memtype.limits.upper.unwrap_or(memtype.limits.lower) <= 65536)
        .true_or(ValidationErrorKind::MemoryTooLarge)?;
    Ok(MemType::new(memtype.limits))
}

fn validate_tabletype(tabletype: TableType<Unvalidated>) -> KindResult<TableType<Validated>> {
    if let Some(upper) = tabletype.limits.upper {
        (tabletype.limits.lower <= upper).true_or(ValidationErrorKind::InvalidLimits)?
    }
    Ok(TableType::new(tabletype.limits, tabletype.reftype))
}

fn validate_import(
    import: ImportField<Resolved, Unvalidated>,
) -> Result<ImportField<Resolved, Validated>> {
    Ok(ImportField {
        id:       import.id,
        modname:  import.modname,
        name:     import.name,
        exports:  import.exports,
        desc:     validate_import_desc(import.desc).validation_error(import.location)?,
        location: import.location,
    })
}

fn validate_import_desc(
    desc: ImportDesc<Resolved, Unvalidated>,
) -> KindResult<ImportDesc<Resolved, Validated>> {
    Ok(match desc {
        ImportDesc::Func(t) => ImportDesc::Func(t),
        ImportDesc::Global(g) => ImportDesc::Global(g),
        ImportDesc::Mem(m) => ImportDesc::Mem(validate_memtype(m)?),
        ImportDesc::Table(t) => ImportDesc::Table(validate_tabletype(t)?),
    })
}

fn compile_export_desc(
    module: &ModuleContext,
    funcrefs: &mut Vec<Index<Resolved, FuncIndex>>,
    exportdesc: ExportDesc<Resolved>,
) -> KindResult<ExportDesc<Resolved>> {
    match exportdesc {
        ExportDesc::Func(fi) => {
            (module.funcs.len() > fi.value() as usize).true_or(ValidationErrorKind::UnknownFunc)?;
            funcrefs.push(fi.clone());
            Ok(ExportDesc::Func(fi))
        }
        ExportDesc::Global(gi) => {
            (module.globals.len() > gi.value() as usize)
                .true_or(ValidationErrorKind::UnknownGlobal)?;
            Ok(ExportDesc::Global(gi))
        }
        ExportDesc::Mem(mi) => {
            (module.mems.len() > mi.value() as usize)
                .true_or(ValidationErrorKind::UnknownMemory)?;
            Ok(ExportDesc::Mem(mi))
        }
        ExportDesc::Table(ti) => {
            (module.tables.len() > ti.value() as usize)
                .true_or(ValidationErrorKind::UnknownTable)?;
            Ok(ExportDesc::Table(ti))
        }
    }
}

fn compile_table_position(
    module: &ModuleContext,
    funcrefs: &mut Vec<Index<Resolved, FuncIndex>>,
    table_position: TablePosition<Resolved, UncompiledExpr<Resolved>>,
    elem_list: ElemList<UncompiledExpr<Resolved>>,
    ei: usize,
    location: &Location,
) -> Result<TablePosition<Resolved, CompiledExpr>> {
    let ti = table_position.tableuse.tableidx.value();
    let cnt = elem_list.items.len();
    let table = module
        .tables
        .get(ti as usize)
        .ok_or(ValidationErrorKind::UnknownTable)
        .validation_error(*location)?;

    (table.reftype == elem_list.reftype)
        .true_or(ValidationErrorKind::TypeMismatch {
            actual: elem_list.reftype.into(),
            expect: table.reftype.into(),
        })
        .validation_error(*location)?;

    let mut offset = compile_const_expr(
        &table_position.offset,
        module,
        funcrefs,
        NumType::I32.into(),
    )
    .validation_error(*location)?;
    let mut offset_expr_instrs = offset.instr.to_vec();

    // Add this to the end:
    // (i32.const 0) (i32.const {cnt}) (table.init {ti} {ei}) (elem.drop {ei})
    offset_expr_instrs.push(0x41);
    offset_expr_instrs.extend(0u32.to_le_bytes());
    offset_expr_instrs.push(0x41);
    offset_expr_instrs.extend((cnt as u32).to_le_bytes());
    offset_expr_instrs.extend(&[0xFC, 0x0C]);
    offset_expr_instrs.extend(ti.to_le_bytes());
    offset_expr_instrs.extend((ei as u32).to_le_bytes());
    offset_expr_instrs.extend(&[0xFC, 0x0D]);
    offset_expr_instrs.extend((ei as u32).to_le_bytes());

    offset.instr = offset_expr_instrs.into_boxed_slice();

    Ok(TablePosition {
        tableuse: table_position.tableuse,
        offset,
    })
}
fn compile_elem_mode(
    module: &ModuleContext,
    funcrefs: &mut Vec<Index<Resolved, FuncIndex>>,
    elem_mode: ModeEntry<Resolved, UncompiledExpr<Resolved>>,
    elem_list: ElemList<UncompiledExpr<Resolved>>,
    ei: usize,
    location: &Location,
) -> Result<ModeEntry<Resolved, CompiledExpr>> {
    Ok(match elem_mode {
        ModeEntry::Active(tp) => ModeEntry::Active(compile_table_position(
            module, funcrefs, tp, elem_list, ei, location,
        )?),
        ModeEntry::Passive => ModeEntry::Passive,
        ModeEntry::Declarative => ModeEntry::Declarative,
    })
}

fn compile_elem_list(
    module: &ModuleContext,
    funcrefs: &mut Vec<Index<Resolved, FuncIndex>>,
    elem_list: &ElemList<UncompiledExpr<Resolved>>,
    location: &Location,
) -> Result<ElemList<CompiledExpr>> {
    Ok(ElemList {
        reftype: elem_list.reftype,
        items:   elem_list
            .items
            .iter()
            .map(|e| {
                compile_const_expr(e, module, funcrefs, elem_list.reftype.into())
                    .validation_error(*location)
            })
            .collect::<Result<Vec<_>>>()?,
    })
}

fn compile_data_init(
    module: &ModuleContext,
    funcrefs: &mut Vec<Index<Resolved, FuncIndex>>,
    data_init: DataInit<Resolved, UncompiledExpr<Resolved>>,
    cnt: usize,
    di: usize,
    location: &Location,
) -> Result<DataInit<Resolved, CompiledExpr>> {
    ((data_init.memidx.value() as usize) < module.mems.len())
        .true_or(ValidationErrorKind::UnknownMemory)
        .validation_error(*location)?;

    let mut offset = compile_const_expr(&data_init.offset, module, funcrefs, NumType::I32.into())
        .validation_error(*location)?;
    let mut offset_expr_instrs = offset.instr.to_vec();

    // "(i32.const 0) (i32.const {cnt}) (memory.init {di}) (data.drop {di})"
    offset_expr_instrs.push(0x41);
    offset_expr_instrs.extend(0u32.to_le_bytes());
    offset_expr_instrs.push(0x41);
    offset_expr_instrs.extend((cnt as u32).to_le_bytes());
    offset_expr_instrs.extend(&[0xFC, 0x08]);
    offset_expr_instrs.extend((di as u32).to_le_bytes());
    offset_expr_instrs.extend(&[0xFC, 0x09]);
    offset_expr_instrs.extend((di as u32).to_le_bytes());

    offset.instr = offset_expr_instrs.into_boxed_slice();

    Ok(DataInit {
        memidx: data_init.memidx,
        offset,
    })
}
