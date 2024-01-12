use {
    super::{
        error::{Result, RuntimeErrorKind},
        instance::{ExportInstance, FunctionInstance, ModuleInstance},
        Runtime,
    },
    crate::{
        log_tag::Tag,
        runtime::{
            instance::{
                module_instance::ModuleInstanceBuilder, DataInstance, ElemInstance, ExternalVal,
                GlobalInstance, MemInstance, TableInstance,
            },
            values::Ref,
        },
        syntax::{
            self,
            types::{FunctionType, ValueType},
            CompiledExpr, DataInit, FuncField, ImportDesc, ModeEntry, Resolved, TablePosition,
        },
    },
    std::{convert::identity, rc::Rc},
    wrausmt_common::{logger::Logger, true_or::TrueOr},
};

impl Runtime {
    /// The load method allocates and instantiates the provided
    /// [syntax::Module].
    pub fn load(
        &mut self,
        module: syntax::Module<Resolved, CompiledExpr>,
    ) -> Result<Rc<ModuleInstance>> {
        self.instantiate(module)
    }

    fn validate_import(
        &self,
        import: &syntax::ImportField<Resolved>,
        candidate_addr: &ExternalVal,
        types: &[FunctionType],
    ) -> Result<()> {
        let matches = match (&import.desc, &candidate_addr) {
            (ImportDesc::Func(fi), ExternalVal::Func(fa)) => {
                let resolved = &self.store.func(*fa)?.functype;
                let imported = &types[fi.index().value() as usize];
                resolved == imported
            }
            (ImportDesc::Table(ti), ExternalVal::Table(ta)) => {
                let resolved = &self.store.table(*ta)?.tabletype;
                resolved.reftype == ti.reftype && resolved.limits.works_as(&ti.limits)
            }
            (ImportDesc::Mem(mi), ExternalVal::Memory(ma)) => {
                let resolved = &self.store.mem(*ma)?;
                resolved.memtype.limits.works_as(&mi.limits)
            }
            (ImportDesc::Global(gi), ExternalVal::Global(ga)) => {
                let existing = self.store.global_inst(*ga)?;
                existing.val.valtype() == gi.valtype && existing.mutable == gi.mutable
            }
            _ => false,
        };
        Ok(matches.true_or_else(|| {
            RuntimeErrorKind::ImportMismatch(import.desc.clone(), *candidate_addr)
        })?)
    }

    fn find_import(
        &self,
        import: &syntax::ImportField<Resolved>,
        types: &[FunctionType],
    ) -> Result<ExternalVal> {
        let regmod = self
            .registered
            .get(&import.modname)
            .ok_or_else(|| RuntimeErrorKind::ModuleNotFound(import.modname.clone()))?;

        let exportinst = regmod.resolve(&import.name).ok_or_else(|| {
            RuntimeErrorKind::ImportNotFound(import.modname.clone(), import.name.clone())
        })?;

        self.validate_import(import, &exportinst.addr, types)?;

        Ok(exportinst.addr)
    }

    /// Instantiate a function from the provided FuncField and module instance.
    fn instantiate_function(
        f: FuncField<Resolved, CompiledExpr>,
        types: &[FunctionType],
        modinst: Rc<ModuleInstance>,
    ) -> Result<FunctionInstance> {
        let functype = types
            .get(f.typeuse.index().value() as usize)
            .ok_or(RuntimeErrorKind::TypeNotFound(f.typeuse.index().value()))?
            .clone();
        // when do params get added?
        let locals: Box<[ValueType]> = f.locals.iter().map(|l| l.valtype).collect();
        Ok(FunctionInstance {
            functype,
            module_instance: modinst,
            locals,
            body: f.body.instr,
        })
    }

    fn init_table(&mut self, tp: &TablePosition<Resolved, CompiledExpr>) -> Result<()> {
        self.exec_expr(&tp.offset.instr)
    }

    fn init_mem(&mut self, datainit: DataInit<Resolved, CompiledExpr>) -> Result<()> {
        self.exec_expr(&datainit.offset.instr)
    }

    fn instantiate_export_desc(
        ast: syntax::ExportDesc<Resolved>,
        modinst: &ModuleInstance,
    ) -> ExternalVal {
        match ast {
            syntax::ExportDesc::Func(idx) => ExternalVal::Func(modinst.func(idx.value())),
            syntax::ExportDesc::Table(idx) => ExternalVal::Table(modinst.table(idx.value())),
            syntax::ExportDesc::Mem(idx) => ExternalVal::Memory(modinst.mem(idx.value())),
            syntax::ExportDesc::Global(idx) => ExternalVal::Global(modinst.global(idx.value())),
        }
    }

    pub fn instantiate_export(
        ast: syntax::ExportField<Resolved>,
        modinst: &ModuleInstance,
    ) -> ExportInstance {
        ExportInstance {
            name: ast.name,
            addr: Self::instantiate_export_desc(ast.exportdesc, modinst),
        }
    }

    fn instantiate(
        &mut self,
        module: syntax::Module<Resolved, CompiledExpr>,
    ) -> Result<Rc<ModuleInstance>> {
        let mut modinst_builder = ModuleInstanceBuilder {
            types: module
                .types
                .into_iter()
                .map(|t| t.functiontype.into())
                .collect(),
            ..ModuleInstanceBuilder::default()
        };

        for import in module.imports {
            let found = self.find_import(&import, &modinst_builder.types)?;
            modinst_builder.add_external_val(found);
        }

        let rcinst = Rc::new(modinst_builder.clone().build());

        // During init, we will reset this a few times.
        // TODO -- maybe there's a better way that avoids unsafe.
        let rcptr = Rc::as_ptr(&rcinst) as *mut ModuleInstance;

        // (Alloc 2.) Allocate functions
        // https://webassembly.github.io/spec/core/exec/modules.html#functions
        // We hold onto these so we can update the module instance at the end.
        let func_insts = module
            .funcs
            .into_iter()
            .map(|f| Self::instantiate_function(f, &modinst_builder.types, rcinst.clone()));

        let range = self.store.alloc(|s| &mut s.funcs, func_insts, Rc::new)?;
        modinst_builder.funcs.extend(range);

        self.logger.log(Tag::Load, || {
            format!("LOADED FUNCTIONS {:?}", modinst_builder.funcs)
        });

        let table_insts = module
            .tables
            .into_iter()
            .map(|t| TableInstance::new(t.tabletype));

        let range = self.store.alloc(|s| &mut s.tables, table_insts, identity)?;
        modinst_builder.tables.extend(range);
        self.logger.log(Tag::Load, || {
            format!("LOADED TABLES {:?}", modinst_builder.tables)
        });

        let mem_insts = module.memories.into_iter().map(MemInstance::new_ast);

        let range = self.store.alloc(|s| &mut s.mems, mem_insts, identity)?;
        modinst_builder.mems.extend(range);
        self.logger.log(Tag::Load, || {
            format!("LOADED MEMS {:?}", modinst_builder.mems)
        });

        // (Instantiation 5-10.) Generate global and elem init values
        // (Instantiation 5.) Create the module instance for global initialization
        unsafe {
            *rcptr = modinst_builder.clone().build();
        }

        // (Instantiation 6-7.) Create a frame with the instance, push it.
        self.stack.push_dummy_activation(rcinst.clone())?;

        // (Instantiation 9.) Elems
        let elem_insts = module
            .elems
            .iter()
            .map(|e| {
                let refs: Box<[Ref]> = match &e.mode {
                    ModeEntry::Declarative => Box::new([]),
                    _ => e
                        .elemlist
                        .items
                        .iter()
                        .map(|ei| self.eval_ref_expr(&ei.instr))
                        .collect::<Result<_>>()?,
                };
                Ok(ElemInstance::new(refs))
            })
            // Since the iterator maps with a closure over self, we need to collect it
            // before we can pass to alloc via self.
            .collect::<Vec<Result<_>>>();
        let range = self
            .store
            .alloc(|s| &mut s.elems, elem_insts.into_iter(), identity)?;
        modinst_builder.elems.extend(range);
        self.logger.log(Tag::Load, || {
            format!("LOADED ELEMS {:?}", modinst_builder.elems)
        });

        let mut data_inits: Vec<DataInit<Resolved, CompiledExpr>> = Vec::new();
        let data_insts = module.data.into_iter().map(|d| {
            // Pluck out the data inits now for step 15.
            if let Some(init) = d.init {
                data_inits.push(init);
            }
            Ok(DataInstance { bytes: d.data })
        });

        let range = self.store.alloc(|s| &mut s.datas, data_insts, identity)?;
        modinst_builder.data.extend(range);
        self.logger.log(Tag::Load, || {
            format!("LOADED DATA {:?}", modinst_builder.data)
        });

        // (Instantiation 8.) Get global init vals and allocate globals.
        let global_insts: Vec<Result<_>> = module
            .globals
            .iter()
            .map(|g| {
                self.logger.log(Tag::Load, || {
                    format!("COMPILE GLOBAL INIT EXPR {:x?}", g.init)
                });
                let val = self.eval_expr(&g.init.instr)?;
                Ok(GlobalInstance {
                    typ: g.globaltype.valtype,
                    mutable: g.globaltype.mutable,
                    val,
                })
            })
            // We have to collect immediately, because the iterator holds a read ref to self.
            // So we can't call self.store.alloc while that's active.
            .collect();

        let range = self
            .store
            .alloc(|s| &mut s.globals, global_insts.into_iter(), identity)?;

        modinst_builder.globals.extend(range);
        self.logger.log(Tag::Load, || {
            format!("LOADED GLOBALS {:?}", modinst_builder.globals)
        });

        // (Instantiation 10.) Pop Finit from the stack.
        self.stack.pop_activation()?;

        // (Instantiation 11, 12.) Create the module instance for global initialization
        // This is OK, nothing should be referencing the old ModuleInstance.
        unsafe {
            *rcptr = modinst_builder.clone().build();
        }

        // (Instantiation 13.) Create a frame with the instance, push it.
        self.stack.push_dummy_activation(rcinst.clone())?;

        // (Instantiation 14.) Active table inits.
        for elem in &module.elems {
            if let ModeEntry::Active(tp) = &elem.mode {
                self.logger
                    .log(Tag::Load, || format!("INIT ELEMS!i {:?}", &elem));
                self.init_table(tp)?
            }
        }

        // (Instantiation 15.) Active mem inits.
        for init in data_inits.into_iter() {
            self.logger
                .log(Tag::Load, || format!("INIT MEMORY !i {:?}", init));
            self.init_mem(init)?
        }

        // This is OK, nothing should be referencing the old ModuleInstance.
        unsafe {
            *rcptr = modinst_builder.clone().build();
        }

        modinst_builder.exports = module
            .exports
            .into_iter()
            .map(|e| Self::instantiate_export(e, &rcinst))
            .collect();

        self.logger.log(Tag::Load, || {
            format!("EXPORTS {:?}", modinst_builder.exports)
        });

        self.stack.pop_activation()?;

        // This is OK, nothing should be referencing the old ModuleInstance.
        unsafe {
            *rcptr = modinst_builder.build();
        }

        if let Some(start) = module.start {
            let startaddr = rcinst.func(start.idx.value());
            self.stack.push_dummy_activation(rcinst.clone())?;
            self.invoke_addr(startaddr)?;
            self.stack.pop_activation()?;
        }

        Ok(rcinst)
    }
}

impl From<syntax::FunctionType> for FunctionType {
    fn from(ast: syntax::FunctionType) -> FunctionType {
        FunctionType {
            params: ast.params.iter().map(|p| p.valuetype).collect(),
            result: ast.results.iter().map(|r| r.valuetype).collect(),
        }
    }
}
