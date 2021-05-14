mod compile;
pub mod error;
pub mod exec;
pub mod instance;
pub mod stack;
pub mod store;
pub mod values;

use crate::error as mkerror;
use crate::{
    err,
    error::{Result, ResultFrom},
    logger::{Logger, PrintLogger},
    runtime::{
        instance::{module_instance::ModuleInstanceBuilder, ElemInstance, TableInstance},
        values::Ref,
    },
    syntax::{self, ElemList, Expr, FuncField, Instruction, ModeEntry, Resolved, TablePosition},
    types::{FunctionType, ValueType},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use {
    compile::{compile_export, compile_function_body, Emitter},
    instance::{
        ExportInstance, ExternalVal, FunctionInstance, GlobalInstance, MemInstance, ModuleInstance,
    },
    stack::Stack,
    store::addr,
    store::Store,
    values::Value,
};

#[derive(Debug, Default)]
/// Contains all of the runtime state for the WebAssembly interpreter.
pub struct Runtime {
    /// The Store of the runtime, as described by the spec.
    store: Store,

    /// The runtime stack.
    stack: Stack,

    /// Modules registered for import
    registered: HashMap<String, Rc<ModuleInstance>>,

    logger: PrintLogger,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime::default()
    }

    /// The load method allocates and instantiates the provided [Module].
    pub fn load(&mut self, module: syntax::Module<Resolved>) -> Result<Rc<ModuleInstance>> {
        self.instantiate(module)
    }

    pub fn register<S: Into<String>>(&mut self, modname: S, module: Rc<ModuleInstance>) {
        self.registered.insert(modname.into(), module);
    }

    fn find_import(&self, modname: &str, name: &str) -> Result<ExternalVal> {
        let regmod = self
            .registered
            .get(modname)
            .ok_or_else(|| mkerror!("No module {}", modname))?;
        let exportinst = regmod
            .resolve(name)
            .ok_or_else(|| mkerror!("No {} in {}", name, modname))?;
        Ok(exportinst.addr)
    }

    /// Instantiate a function from the provided FuncField and module instance.
    fn instantiate_function(f: FuncField<Resolved>, types: &[FunctionType]) -> FunctionInstance {
        let functype = types[f.typeuse.index_value() as usize].clone();
        let locals: Box<[ValueType]> = f.locals.iter().map(|l| l.valtype).collect();
        let body = compile_function_body(&f);
        FunctionInstance {
            functype,
            module_instance: RefCell::new(None),
            locals,
            body,
        }
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
        // TODO - offset has and end marker 0x0b throwing off label count.
        init_code.emit_expr(&tp.offset);
        self.exec_expr(&init_code)?;
        init_code.clear();
        init_code.emit_expr(&Expr { instr: initexpr });
        self.exec_expr(&init_code)?;
        Ok(())
    }

    fn extend_addr_vec(vec: &mut Vec<u32>, range: std::ops::Range<u32>) {
        for i in range {
            vec.push(i);
        }
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

        // TODO - actually resolve imports
        for import in module.imports {
            let found = self.find_import(&import.modname, &import.name)?;

            match (&import.desc, found) {
                (syntax::ImportDesc::Func(_), ExternalVal::Func(addr)) => {
                    modinst_builder.funcs.push(addr);
                }
                (syntax::ImportDesc::Table(_), ExternalVal::Table(addr)) => {
                    modinst_builder.tables.push(addr);
                }
                (syntax::ImportDesc::Mem(_), ExternalVal::Memory(addr)) => {
                    modinst_builder.mems.push(addr);
                }
                (syntax::ImportDesc::Global(_), ExternalVal::Global(addr)) => {
                    modinst_builder.globals.push(addr);
                }
                _ => return err!("Wrong export type {:?} for {:?}", found, import),
            }
        }

        // (Alloc 2.) Allocate functions
        // https://webassembly.github.io/spec/core/exec/modules.html#functions
        // We hold onto these so we can update the module instance at the end.
        let func_insts: Vec<Rc<FunctionInstance>> = module
            .funcs
            .into_iter()
            .map(|f| Self::instantiate_function(f, &modinst_builder.types))
            .map(Rc::new)
            .collect();
        let range = self.store.alloc_funcs(func_insts.iter().cloned());
        Self::extend_addr_vec(&mut modinst_builder.funcs, range);

        self.logger.log("LOAD", || {
            format!("LOADED FUNCTIONS {:?}", modinst_builder.funcs)
        });

        let table_insts: Vec<TableInstance> = module
            .tables
            .into_iter()
            .map(|t| TableInstance::new(t.tabletype))
            .collect();
        let range = self.store.alloc_tables(table_insts.into_iter());
        Self::extend_addr_vec(&mut modinst_builder.tables, range);
        self.logger.log("LOAD", || {
            format!("LOADED TABLES {:?}", modinst_builder.tables)
        });

        let mem_insts = module.memories.into_iter().map(MemInstance::new_ast);
        let range = self.store.alloc_mems(mem_insts);
        Self::extend_addr_vec(&mut modinst_builder.mems, range);
        self.logger
            .log("LOAD", || format!("LOADED MEMS {:?}", modinst_builder.mems));

        // (Instantiation 5-10.) Generate global and elem init values
        // (Instantiation 5.) Create the module instance for global initialization
        let init_module_instance = Rc::new(modinst_builder.clone().build());

        // (Instantiation 6-7.) Create a frame with the instance, push it.
        self.stack.push_dummy_activation(init_module_instance)?;

        // (Instantiation 9.) Elems
        let elem_insts: Vec<ElemInstance> = module
            .elems
            .iter()
            .map(|e| {
                let refs: Vec<Ref> = e
                    .elemlist
                    .items
                    .iter()
                    .map(|ei| {
                        let mut initexpr: Vec<u8> = Vec::new();
                        initexpr.emit_expr(&ei);
                        self.eval_ref_expr(&initexpr)
                    })
                    .collect::<Result<_>>()?;
                Ok(ElemInstance::new(refs.into_boxed_slice()))
            })
            .collect::<Result<_>>()?;
        let range = self.store.alloc_elems(elem_insts.into_iter());
        Self::extend_addr_vec(&mut modinst_builder.elems, range);
        self.logger.log("LOAD", || {
            format!("LOADED ELEMS {:?}", modinst_builder.elems)
        });

        // (Instantiation 8.) Get global init vals and allocate globals.
        let global_insts: Vec<GlobalInstance> = module
            .globals
            .iter()
            .map(|g| {
                self.logger
                    .log("LOAD", || format!("COMPILE GLOBAL INIT EXPR {:x?}", g.init));
                let mut initexpr: Vec<u8> = Vec::new();
                initexpr.emit_expr(&g.init);
                let val = self.eval_expr(&initexpr).wrap("initializing global")?;
                Ok(GlobalInstance {
                    typ: g.globaltype.valtype,
                    val,
                })
            })
            .collect::<Result<_>>()?;
        let range = self.store.alloc_globals(global_insts.into_iter());
        Self::extend_addr_vec(&mut modinst_builder.globals, range);
        self.logger.log("LOAD", || {
            format!("LOADED GLOBALS {:?}", modinst_builder.globals)
        });

        // (Instantiation 10.) Pop Finit from the stack.
        self.stack.pop_activation()?;

        // (Instantiation 11, 12.) Create the module instance for global initialization
        let init_module_instance = Rc::new(modinst_builder.clone().build());

        // (Instantiation 13.) Create a frame with the instance, push it.
        self.stack.push_dummy_activation(init_module_instance)?;

        // (Instantiation 14.) Active table inits.
        for (i, elem) in module.elems.iter().enumerate() {
            if let ModeEntry::Active(tp) = &elem.mode {
                self.logger
                    .log("LOAD", || format!("INIT ELEMS!i {:?}", elem));
                self.init_table(tp, &elem.elemlist, i as u32)?
            }
        }

        let init_module_instance = Rc::new(modinst_builder.clone().build());
        modinst_builder.exports = module
            .exports
            .into_iter()
            .map(|e| compile_export(e, &init_module_instance))
            .collect();

        self.logger
            .log("LOAD", || format!("EXPORTS {:?}", modinst_builder.exports));

        self.stack.pop_activation()?;

        let rcinst = Rc::new(modinst_builder.build());

        // As noted in the specification for module allocation: functions are defined before the
        // final [ModuleInstance] is available, so now we pass the completed instance to the store
        // so it can update the value.
        for f in func_insts {
            f.module_instance.replace(Some(rcinst.clone()));
        }

        Ok(rcinst)
    }

    pub fn invoke(&mut self, addr: addr::FuncAddr) -> Result<()> {
        // 1. Assert S.funcaddr exists
        // 2. Let funcinst = S.funcs[funcaddr]
        let funcinst = self.store.func(addr)?;

        // 3. Let [tn_1] -> [tm_2] be the function type.
        // 4. Let t* be the list of locals.
        // 5. Let instr* end be the code body
        // 6. Assert (due to validation) n values on the stack
        // 7. Pop val_n from the stack
        // 8. Let val0* be the list of zero values (other locals).
        // 9. Let F be the frame.
        // 10. Push activation w/ arity m onto the stack.
        self.stack.push_activation(&funcinst)?;

        // 11. Let L be the Label with continuation at function end.
        // 12. Enter the instruction sequence with the label.
        let arity = funcinst.functype.result.len() as u32;
        let continuation = funcinst.body.len() as u32;

        self.stack.push_label(0, arity, continuation)?;

        self.enter(&funcinst.body)?;

        // NOTE: The compiled function has an `end` instruction at the end
        // which takes care of popping the label.

        // Due to validation, this should be equal to the frame above.
        self.stack.pop_activation()?;

        self.logger.log("ACTIVATION", || {
            format!(
                "REMOVE FRAME {} {} {}",
                funcinst.locals.len(),
                funcinst.functype.params.len(),
                funcinst.functype.result.len(),
            )
        });

        Ok(())
    }

    /// Invocation of a function by the host.
    pub fn call(
        &mut self,
        mod_instance: &Rc<ModuleInstance>,
        name: &str,
        vals: &[Value],
    ) -> Result<Vec<Value>> {
        let funcaddr = match mod_instance.resolve(name) {
            Some(ExportInstance {
                name: _,
                addr: ExternalVal::Func(idx),
            }) => Ok(*idx),
            _ => err!("Method not found in module: {}", name),
        }?;

        self.logger
            .log("HOST", || format!("calling {} at {}", name, funcaddr));
        // 1. Assert S.funcaddr exists
        // 2. Let funcinst = S.funcs[funcaddr]
        let funcinst = self
            .store
            .func(funcaddr)
            .wrap(&format!("for name {}", name))?;

        // 3. Let [tn_1] -> [tm_2] be the function type.
        // 4. If the length of vals is different then the number of vals provided, fail.
        // 5. For each value type, if not matching declared type, fail.
        funcinst
            .validate_args(vals)
            .wrap(&format!("for {}", name))?;

        // 6. Let F be a dummy frame. (Represents a dummy "caller" for the function to invoke).
        // 7. Push F to the stack.
        self.stack.push_dummy_activation(mod_instance.clone())?;

        // 8. Push the values to the stack.
        for val in vals {
            self.stack.push_value(*val);
        }

        // 9. Invoke the function.
        self.invoke(funcaddr)?;

        let mut results: Vec<Value> = vec![];
        for i in 0..funcinst.functype.result.len() {
            let result = self
                .stack
                .pop_value()
                .wrap(&format!("popping result {} for {}", i, name))?;
            self.logger
                .log("HOST", || format!("POPPED HOST RESULT {:?}", result));
            results.push(result);
        }

        // pop the dummy frame
        // due to validation, this will be the one we pushed above.
        self.stack.pop_activation()?;

        // Since we don't do validation yet, do some checking here to make sure things seem ok.
        if let Ok(v) = self.stack.pop_value() {
            return err!("values still on stack {:?}", v);
        }

        if let Ok(l) = self.stack.peek_label() {
            return err!("labels still on stack {:?}", l);
        }

        if self.stack.activation_depth() != 0 {
            return err!("frames still on stack");
        }
        Ok(results)
    }
}

#[macro_export]
macro_rules! runner {
    ( $runtime:expr, $mod_inst:expr ) => {
        macro_rules! exec_method {
            ( $cmd:expr ) => {
                $runtime.call(&$mod_inst, $cmd, &[]);
            };
            ( $cmd:expr, $v1:expr ) => {
                $runtime.call(&$mod_inst, $cmd, &[$v1.into()]);
            };
            ( $cmd:expr, $v1:expr, $v2:expr ) => {
                $runtime.call(&$mod_inst, $cmd, &[$v1.into(), $v2.into()]);
            };
        }
    };
}
