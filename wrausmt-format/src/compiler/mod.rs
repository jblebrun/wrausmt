mod emitter;
mod validation;
pub use validation::{ValidationError, ValidationErrorKind, ValidationMode};
use {
    self::{emitter::ValidatingEmitter, validation::ModuleContext},
    validation::{KindResult, Result},
    wrausmt_common::true_or::TrueOr,
    wrausmt_runtime::syntax::{
        location::Location,
        types::{NumType, ValueType},
        CompiledExpr, DataField, DataInit, ElemField, ElemList, FuncField, GlobalField, ModeEntry,
        Module, Resolved, TablePosition, UncompiledExpr,
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
        location: func.location,
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
            &global.location,
            validation::ExpressionType::Constant,
        )?,
        location:   global.location,
    })
}

fn compile_elem(
    validation_mode: ValidationMode,
    module: &ModuleContext,
    elem: ElemField<Resolved, UncompiledExpr<Resolved>>,
    ei: usize,
) -> Result<ElemField<Resolved, CompiledExpr>> {
    let elemlist = compile_elem_list(validation_mode, module, &elem.elemlist, &elem.location)?;
    Ok(ElemField {
        id: elem.id,
        mode: compile_elem_mode(
            validation_mode,
            module,
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
    validation_mode: ValidationMode,
    module: &ModuleContext,
    data: DataField<Resolved, UncompiledExpr<Resolved>>,
    di: usize,
) -> Result<DataField<Resolved, CompiledExpr>> {
    let dlen = data.data.len();
    Ok(DataField {
        id:       data.id,
        data:     data.data,
        init:     match data.init {
            Some(data_init) => Some(compile_data_init(
                validation_mode,
                module,
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

fn compile_table_position(
    validation_mode: ValidationMode,
    module: &ModuleContext,
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

    let mut offset = ValidatingEmitter::simple_expression(
        validation_mode,
        module,
        &table_position.offset,
        vec![ValueType::Num(NumType::I32)],
        location,
        validation::ExpressionType::Constant,
    )?;
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
    validation_mode: ValidationMode,
    module: &ModuleContext,
    elem_mode: ModeEntry<Resolved, UncompiledExpr<Resolved>>,
    elem_list: ElemList<UncompiledExpr<Resolved>>,
    ei: usize,
    location: &Location,
) -> Result<ModeEntry<Resolved, CompiledExpr>> {
    Ok(match elem_mode {
        ModeEntry::Active(tp) => ModeEntry::Active(compile_table_position(
            validation_mode,
            module,
            tp,
            elem_list,
            ei,
            location,
        )?),
        ModeEntry::Passive => ModeEntry::Passive,
        ModeEntry::Declarative => ModeEntry::Declarative,
    })
}

fn compile_elem_list(
    validation_mode: ValidationMode,
    module: &ModuleContext,
    elem_list: &ElemList<UncompiledExpr<Resolved>>,
    location: &Location,
) -> Result<ElemList<CompiledExpr>> {
    Ok(ElemList {
        reftype: elem_list.reftype,
        items:   elem_list
            .items
            .iter()
            .map(|e| {
                ValidatingEmitter::simple_expression(
                    validation_mode,
                    module,
                    e,
                    vec![elem_list.reftype.into()],
                    location,
                    validation::ExpressionType::Constant,
                )
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
    location: &Location,
) -> Result<DataInit<Resolved, CompiledExpr>> {
    let mut offset = ValidatingEmitter::simple_expression(
        validation_mode,
        module,
        &data_init.offset,
        vec![ValueType::Num(NumType::I32)],
        location,
        validation::ExpressionType::Constant,
    )?;
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
