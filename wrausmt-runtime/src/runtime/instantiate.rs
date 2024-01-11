use {
    super::{
        compile::{compile_function_body, compile_simple_expression},
        error::{Result, RuntimeErrorKind},
        instance::{FunctionInstance, ModuleInstance},
        Runtime,
    },
    crate::{
        log_tag::Tag,
        runtime::{
            compile::compile_export,
            instance::{
                module_instance::ModuleInstanceBuilder, DataInstance, ElemInstance, ExternalVal,
                GlobalInstance, MemInstance, TableInstance,
            },
            values::Ref,
        },
        syntax::{
            self,
            types::{FunctionType, ValueType},
            DataInit, ElemList, FuncField, ImportDesc, Instruction, ModeEntry, Resolved,
            TablePosition, UncompiledExpr,
        },
        validation::ValidationMode,
    },
    std::{convert::identity, rc::Rc},
    wrausmt_common::{logger::Logger, true_or::TrueOr},
};

impl Runtime {
    /// The load method allocates and instantiates the provided
    /// [syntax::Module].
    pub fn load(
        &mut self,
        module: syntax::Module<Resolved, UncompiledExpr<Resolved>>,
        validation_mode: ValidationMode,
    ) -> Result<Rc<ModuleInstance>> {
        self.instantiate(module, validation_mode)
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
        f: FuncField<Resolved, UncompiledExpr<Resolved>>,
        types: &[FunctionType],
        modinst: Rc<ModuleInstance>,
        validation_mode: ValidationMode,
    ) -> Result<FunctionInstance> {
        let functype = types
            .get(f.typeuse.index().value() as usize)
            .ok_or(RuntimeErrorKind::TypeNotFound(f.typeuse.index().value()))?
            .clone();
        let body = compile_function_body(validation_mode, &f, &functype, &modinst)?;
        // when do params get added?
        let locals: Box<[ValueType]> = f.locals.iter().map(|l| l.valtype).collect();
        Ok(FunctionInstance {
            functype,
            module_instance: modinst,
            locals,
            body,
        })
    }

    fn init_table(
        &mut self,
        tp: &TablePosition<Resolved, UncompiledExpr<Resolved>>,
        elemlist: &ElemList<UncompiledExpr<Resolved>>,
        ei: u32,
        validation_mode: ValidationMode,
        modinst: &ModuleInstance,
    ) -> Result<()> {
        let n = elemlist.items.len() as u32;
        let ti = tp.tableuse.tableidx.value();
        let initexpr = UncompiledExpr {
            instr: vec![
                Instruction::i32const(0),
                Instruction::i32const(n),
                Instruction::tableinit(ti, ei),
                Instruction::elemdrop(ei),
            ],
        };
        // TODO can these be combined?
        let init_code = compile_simple_expression(validation_mode, &tp.offset, modinst)?;
        self.exec_expr(&init_code)?;
        let init_code = compile_simple_expression(validation_mode, &initexpr, modinst)?;
        self.exec_expr(&init_code)
    }

    fn init_mem(
        &mut self,
        datainit: DataInit<Resolved, UncompiledExpr<Resolved>>,
        n: u32,
        di: u32,
        validation_mode: ValidationMode,
        modinst: &ModuleInstance,
    ) -> Result<()> {
        let initexpr = UncompiledExpr {
            instr: vec![
                Instruction::i32const(0),
                Instruction::i32const(n),
                Instruction::meminit(di),
                Instruction::datadrop(di),
            ],
        };
        // TODO can these be combined?
        let init_code = compile_simple_expression(validation_mode, &datainit.offset, modinst)?;
        self.exec_expr(&init_code)?;
        let init_code = compile_simple_expression(validation_mode, &initexpr, modinst)?;
        self.exec_expr(&init_code)
    }

    fn instantiate(
        &mut self,
        module: syntax::Module<Resolved, UncompiledExpr<Resolved>>,
        validation_mode: ValidationMode,
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
        let func_insts = module.funcs.into_iter().map(|f| {
            Self::instantiate_function(f, &modinst_builder.types, rcinst.clone(), validation_mode)
        });

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
                        .map(|ei| {
                            let initexpr = compile_simple_expression(validation_mode, ei, &rcinst)?;
                            self.eval_ref_expr(&initexpr)
                        })
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

        let (data_inits, data_insts): (Vec<_>, Vec<_>) = module
            .data
            .into_iter()
            .map(|d| ((d.init, d.data.len()), Ok(DataInstance { bytes: d.data })))
            .unzip();

        let range = self
            .store
            .alloc(|s| &mut s.datas, data_insts.into_iter(), identity)?;
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
                let initexpr = compile_simple_expression(validation_mode, &g.init, &rcinst)?;
                let val = self.eval_expr(&initexpr)?;
                Ok(GlobalInstance {
                    typ: g.globaltype.valtype,
                    mutable: g.globaltype.mutable,
                    val,
                })
            })
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
        for (i, elem) in module.elems.iter().enumerate() {
            if let ModeEntry::Active(tp) = &elem.mode {
                self.logger
                    .log(Tag::Load, || format!("INIT ELEMS!i {:?}", &elem));
                self.init_table(tp, &elem.elemlist, i as u32, validation_mode, &rcinst)?
            }
        }

        // (Instantiation 15.) Active mem inits.
        for (i, initrec) in data_inits.into_iter().enumerate() {
            if let Some(init) = initrec.0 {
                self.logger
                    .log(Tag::Load, || format!("INIT MEMORY !i {:?}", init));
                self.init_mem(init, initrec.1 as u32, i as u32, validation_mode, &rcinst)?
            }
        }

        // This is OK, nothing should be referencing the old ModuleInstance.
        unsafe {
            *rcptr = modinst_builder.clone().build();
        }

        modinst_builder.exports = module
            .exports
            .into_iter()
            .map(|e| compile_export(e, &rcinst))
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
