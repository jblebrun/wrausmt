mod emitter;
mod validation;
pub use validation::{ValidationError, ValidationMode};
use {
    self::{emitter::ValidatingEmitter, validation::ModuleContext},
    validation::Result,
    wrausmt_runtime::syntax::{
        CompiledExpr, DataField, DataInit, ElemField, ElemList, FuncField, GlobalField,
        Instruction, ModeEntry, Module, Resolved, TablePosition, UncompiledExpr,
    },
};

// Compiles all functions in the module.
// It will consume the provided module, so you should clone the module if you
// need to do anything else with it later.
pub fn compile_module(
    validation_mode: ValidationMode,
    module: Module<Resolved, UncompiledExpr<Resolved>>,
) -> Result<Module<Resolved, CompiledExpr>> {
    // We need to create this now and hold onto it, beacuse the module will
    // change as we process its elements.
    let module_context = ModuleContext::new(&module);

    let mut module = module;

    let funcs: Result<Vec<_>> = std::mem::take(&mut module.funcs)
        .into_iter()
        .map(|f| compile_func(validation_mode, &module_context, f))
        .collect();
    let funcs = funcs?;

    let globals: Result<Vec<_>> = std::mem::take(&mut module.globals)
        .into_iter()
        .map(|g| compile_global(validation_mode, &module_context, g))
        .collect();
    let globals = globals?;

    let elems: Result<Vec<_>> = std::mem::take(&mut module.elems)
        .into_iter()
        .enumerate()
        .map(|(i, e)| compile_elem(validation_mode, &module_context, e, i))
        .collect();
    let elems = elems?;

    let data: Result<Vec<_>> = std::mem::take(&mut module.data)
        .into_iter()
        .enumerate()
        .map(|(i, d)| compile_data(validation_mode, &module_context, d, i))
        .collect();
    let data = data?;

    Ok(Module {
        id: module.id,
        customs: module.customs,
        types: module.types,
        funcs,
        tables: module.tables,
        memories: module.memories,
        imports: module.imports,
        exports: module.exports,
        globals,
        start: module.start,
        elems,
        data,
    })
}

fn compile_func(
    validation_mode: ValidationMode,
    module: &ModuleContext,
    func: FuncField<Resolved, UncompiledExpr<Resolved>>,
) -> Result<FuncField<Resolved, CompiledExpr>> {
    let body = ValidatingEmitter::function_body(validation_mode, module, &func)?;
    Ok(FuncField {
        id: func.id,
        exports: func.exports,
        typeuse: func.typeuse,
        locals: func.locals,
        body,
    })
}

fn compile_global(
    validation_mode: ValidationMode,
    module: &ModuleContext,
    global: GlobalField<UncompiledExpr<Resolved>>,
) -> Result<GlobalField<CompiledExpr>> {
    let expect_type = global.globaltype.valtype;
    Ok(GlobalField {
        id:         global.id,
        exports:    global.exports,
        globaltype: global.globaltype,
        init:       ValidatingEmitter::simple_expression(
            validation_mode,
            module,
            &global.init,
            vec![expect_type],
        )?,
    })
}

fn compile_elem(
    validation_mode: ValidationMode,
    module: &ModuleContext,
    elem: ElemField<Resolved, UncompiledExpr<Resolved>>,
    ei: usize,
) -> Result<ElemField<Resolved, CompiledExpr>> {
    Ok(ElemField {
        id:       elem.id,
        mode:     compile_elem_mode(
            validation_mode,
            module,
            elem.mode,
            elem.elemlist.items.len(),
            ei,
        )?,
        elemlist: compile_elem_list(validation_mode, module, elem.elemlist)?,
    })
}

fn compile_data(
    validation_mode: ValidationMode,
    module: &ModuleContext,
    data: DataField<Resolved, UncompiledExpr<Resolved>>,
    di: usize,
) -> Result<DataField<Resolved, CompiledExpr>> {
    let dlen = data.data.len();
    Ok(DataField {
        id:   data.id,
        data: data.data,
        init: match data.init {
            Some(data_init) => Some(compile_data_init(
                validation_mode,
                module,
                data_init,
                dlen,
                di,
            )?),
            None => None,
        },
    })
}

fn compile_table_position(
    validation_mode: ValidationMode,
    module: &ModuleContext,
    table_position: TablePosition<Resolved, UncompiledExpr<Resolved>>,
    cnt: usize,
    ei: usize,
) -> Result<TablePosition<Resolved, CompiledExpr>> {
    let ti = table_position.tableuse.tableidx.value();
    let init_expr = UncompiledExpr {
        instr: vec![
            Instruction::i32const(0),
            Instruction::i32const(cnt as u32),
            Instruction::tableinit(ti, ei as u32),
            Instruction::elemdrop(ei as u32),
        ],
    };
    Ok(TablePosition {
        tableuse: table_position.tableuse,
        offset:   ValidatingEmitter::simple_expressions(
            validation_mode,
            module,
            &[&table_position.offset, &init_expr],
            vec![],
        )?,
    })
}
fn compile_elem_mode(
    validation_mode: ValidationMode,
    module: &ModuleContext,
    elem_mode: ModeEntry<Resolved, UncompiledExpr<Resolved>>,
    cnt: usize,
    ei: usize,
) -> Result<ModeEntry<Resolved, CompiledExpr>> {
    Ok(match elem_mode {
        ModeEntry::Active(tp) => ModeEntry::Active(compile_table_position(
            validation_mode,
            module,
            tp,
            cnt,
            ei,
        )?),
        ModeEntry::Passive => ModeEntry::Passive,
        ModeEntry::Declarative => ModeEntry::Declarative,
    })
}

fn compile_elem_list(
    validation_mode: ValidationMode,
    module: &ModuleContext,
    elem_list: ElemList<UncompiledExpr<Resolved>>,
) -> Result<ElemList<CompiledExpr>> {
    Ok(ElemList {
        reftype: elem_list.reftype,
        items:   elem_list
            .items
            .iter()
            .map(|e| {
                ValidatingEmitter::simple_expression(validation_mode, module, e, vec![elem_list
                    .reftype
                    .into()])
            })
            .collect::<Result<Vec<_>>>()?,
    })
}

fn compile_data_init(
    validation_mode: ValidationMode,
    module: &ModuleContext,
    data_init: DataInit<Resolved, UncompiledExpr<Resolved>>,
    cnt: usize,
    di: usize,
) -> Result<DataInit<Resolved, CompiledExpr>> {
    let init_expr = UncompiledExpr {
        instr: vec![
            Instruction::i32const(0),
            Instruction::i32const(cnt as u32),
            Instruction::meminit(di as u32),
            Instruction::datadrop(di as u32),
        ],
    };
    Ok(DataInit {
        memidx: data_init.memidx,
        offset: ValidatingEmitter::simple_expressions(
            validation_mode,
            module,
            &[&data_init.offset, &init_expr],
            vec![],
        )?,
    })
}
