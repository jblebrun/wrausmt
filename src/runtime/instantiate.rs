use {
    super::{
        compile::compile_function_body,
        error::{Result, RuntimeErrorKind},
        instance::{FunctionInstance, ModuleInstance},
        Runtime,
    },
    crate::{
        logger::{Logger, Tag},
        runtime::{
            compile::{compile_export, Emitter},
            instance::{
                module_instance::ModuleInstanceBuilder, DataInstance, ElemInstance, ExternalVal,
                GlobalInstance, MemInstance, TableInstance,
            },
            values::Ref,
        },
        syntax::{
            self, DataInit, ElemList, Expr, FuncField, ImportDesc, Instruction, ModeEntry,
            Resolved, TablePosition,
        },
        types::{FunctionType, ValueType},
    },
    std::rc::Rc,
};

impl Runtime {
    /// The load method allocates and instantiates the provided
    /// [syntax::Module].
    pub fn load(&mut self, module: syntax::Module<Resolved>) -> Result<Rc<ModuleInstance>> {
        self.instantiate(module)
    }

    fn find_import(&self, import: &syntax::ImportField<Resolved>) -> Result<ExternalVal> {
        let regmod = self
            .registered
            .get(&import.modname)
            .ok_or_else(|| RuntimeErrorKind::ModuleNotFound(import.modname.clone()).error())?;

        let exportinst = regmod.resolve(&import.name).ok_or_else(|| {
            RuntimeErrorKind::ImportNotFound(import.modname.clone(), import.name.clone()).error()
        })?;
        match (&import.desc, &exportinst.addr) {
            (ImportDesc::Func(_), ExternalVal::Func(_)) => (),
            (ImportDesc::Table(_), ExternalVal::Table(_)) => (),
            (ImportDesc::Mem(_), ExternalVal::Memory(_)) => (),
            (ImportDesc::Global(_), ExternalVal::Global(_)) => (),
            _ => {
                return Err(
                    RuntimeErrorKind::ImportMismatch(import.desc.clone(), exportinst.addr).error(),
                )
            }
        };
        Ok(exportinst.addr)
    }

    /// Instantiate a function from the provided FuncField and module instance.
    fn instantiate_function(
        f: FuncField<Resolved>,
        types: &[FunctionType],
        modinst: Rc<ModuleInstance>,
    ) -> Result<FunctionInstance> {
        let functype = types
            .get(f.typeuse.index_value() as usize)
            .ok_or(RuntimeErrorKind::TypeNotFound(f.typeuse.index_value()).error())?
            .clone();
        let locals: Box<[ValueType]> = f.locals.iter().map(|l| l.valtype).collect();
        let body = compile_function_body(&f);
        Ok(FunctionInstance {
            functype,
            module_instance: modinst,
            locals,
            body,
        })
    }

    fn init_table(
        &mut self,
        tp: &TablePosition<Resolved>,
        elemlist: &ElemList<Resolved>,
        ei: u32,
    ) -> Result<()> {
        let n = elemlist.items.len() as u32;
        let ti = tp.tableuse.tableidx.value();
        let initexpr: Vec<Instruction<Resolved>> = vec![
            Instruction::i32const(0),
            Instruction::i32const(n),
            Instruction::tableinit(ti, ei),
            Instruction::elemdrop(ei),
        ];
        let mut init_code: Vec<u8> = vec![];
        init_code.emit_expr(&tp.offset);
        self.exec_expr(&init_code)?;
        init_code.clear();
        init_code.emit_expr(&Expr { instr: initexpr });
        self.exec_expr(&init_code)?;
        Ok(())
    }

    fn init_mem(&mut self, datainit: &DataInit<Resolved>, n: u32, di: u32) -> Result<()> {
        let initexpr: Vec<Instruction<Resolved>> = vec![
            Instruction::i32const(0),
            Instruction::i32const(n),
            Instruction::meminit(di),
            Instruction::datadrop(di),
        ];
        let mut init_code: Vec<u8> = vec![];
        init_code.emit_expr(&datainit.offset);
        self.exec_expr(&init_code)?;
        init_code.clear();
        init_code.emit_expr(&Expr { instr: initexpr });
        self.exec_expr(&init_code)?;
        Ok(())
    }

    fn instantiate(&mut self, module: syntax::Module<Resolved>) -> Result<Rc<ModuleInstance>> {
        let mut modinst_builder = ModuleInstanceBuilder {
            types: module
                .types
                .into_iter()
                .map(|t| t.functiontype.into())
                .collect(),
            ..ModuleInstanceBuilder::default()
        };

        for import in module.imports {
            let found = self.find_import(&import)?;
            modinst_builder.add_external_val(found);
        }

        let rcinst = Rc::new(modinst_builder.clone().build());

        // During init, we will reset this a few times.
        let rcptr = Rc::as_ptr(&rcinst) as *mut ModuleInstance;

        // (Alloc 2.) Allocate functions
        // https://webassembly.github.io/spec/core/exec/modules.html#functions
        // We hold onto these so we can update the module instance at the end.
        let func_insts: Vec<FunctionInstance> = module
            .funcs
            .into_iter()
            .map(|f| Self::instantiate_function(f, &modinst_builder.types, rcinst.clone()))
            .collect::<Result<Vec<FunctionInstance>>>()?;

        let range = self.store.alloc_funcs(func_insts);
        modinst_builder.funcs.extend(range);

        self.logger.log(Tag::Load, || {
            format!("LOADED FUNCTIONS {:?}", modinst_builder.funcs)
        });

        let table_insts: Vec<TableInstance> = module
            .tables
            .into_iter()
            .map(|t| TableInstance::new(t.tabletype))
            .collect();
        let range = self.store.alloc_tables(table_insts.into_iter());
        modinst_builder.tables.extend(range);
        self.logger.log(Tag::Load, || {
            format!("LOADED TABLES {:?}", modinst_builder.tables)
        });

        let mem_insts = module.memories.into_iter().map(MemInstance::new_ast);
        let range = self.store.alloc_mems(mem_insts);
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
        let elem_insts: Vec<ElemInstance> = module
            .elems
            .iter()
            .map(|e| {
                let refs: Vec<Ref> = match e.mode {
                    ModeEntry::Declarative => vec![],
                    _ => e
                        .elemlist
                        .items
                        .iter()
                        .map(|ei| {
                            let mut initexpr: Vec<u8> = Vec::new();
                            initexpr.emit_expr(ei);
                            self.eval_ref_expr(&initexpr)
                        })
                        .collect::<Result<_>>()?,
                };
                Ok(ElemInstance::new(refs.into_boxed_slice()))
            })
            .collect::<Result<_>>()?;
        let range = self.store.alloc_elems(elem_insts.into_iter());
        modinst_builder.elems.extend(range);
        self.logger.log(Tag::Load, || {
            format!("LOADED ELEMS {:?}", modinst_builder.elems)
        });

        let (data_inits, data_insts): (Vec<_>, Vec<_>) = module
            .data
            .into_iter()
            .map(|d| ((d.init, d.data.len()), DataInstance { bytes: d.data }))
            .unzip();

        let range = self.store.alloc_data(data_insts.into_iter());
        modinst_builder.data.extend(range);
        self.logger.log(Tag::Load, || {
            format!("LOADED DATA {:?}", modinst_builder.data)
        });

        // (Instantiation 8.) Get global init vals and allocate globals.
        let global_insts: Vec<GlobalInstance> = module
            .globals
            .iter()
            .map(|g| {
                self.logger.log(Tag::Load, || {
                    format!("COMPILE GLOBAL INIT EXPR {:x?}", g.init)
                });
                let mut initexpr: Vec<u8> = Vec::new();
                initexpr.emit_expr(&g.init);
                let val = self.eval_expr(&initexpr)?;
                Ok(GlobalInstance {
                    typ: g.globaltype.valtype,
                    val,
                })
            })
            .collect::<Result<_>>()?;
        let range = self.store.alloc_globals(global_insts.into_iter());
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
                    .log(Tag::Load, || format!("INIT ELEMS!i {:?}", elem));
                self.init_table(tp, &elem.elemlist, i as u32)?
            }
        }

        // (Instantiation 15.) Active mem inits.
        for (i, initrec) in data_inits.iter().enumerate() {
            if let Some(init) = &initrec.0 {
                self.logger
                    .log(Tag::Load, || format!("INIT MEMORY !i {:?}", init));
                self.init_mem(init, initrec.1 as u32, i as u32)?
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
