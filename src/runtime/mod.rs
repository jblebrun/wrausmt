pub mod error;
pub mod exec;
pub mod instance;
pub mod stack;
pub mod store;
pub mod values;

use crate::{
    err,
    error::{Result, ResultFrom},
    format::text::compile::{compile_export, compile_function_body, Emitter},
    syntax::{self, FuncField, Resolved},
    types::ValueType,
};
use std::{cell::RefCell, rc::Rc};

use {
    instance::{
        ExportInstance, ExternalVal, FunctionInstance, GlobalInstance, MemInstance, ModuleInstance,
    },
    stack::{Label, Stack},
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
}

impl Runtime {
    pub fn new() -> Self {
        Runtime::default()
    }

    /// The load method allocates and instantiates the provided [Module].
    pub fn load(&mut self, module: syntax::Module<Resolved>) -> Result<Rc<ModuleInstance>> {
        // TODO Resolve imports
        for import in module.imports.iter() {
            println!("NEED TO RESOLVE {}:{}", import.modname, import.name);
        }
        self.instantiate(module)
    }

    /// Instantiate a function from the provided FuncField and module instance.
    fn instantiate_function(f: FuncField<Resolved>, modinst: &ModuleInstance) -> FunctionInstance {
        let functype = modinst.types[f.typeuse.index_value() as usize].clone();
        let locals: Box<[ValueType]> = f.locals.iter().map(|l| l.valtype).collect();
        let body = compile_function_body(&f);
        FunctionInstance {
            functype,
            module_instance: RefCell::new(None),
            locals,
            body,
        }
    }

    fn instantiate(&mut self, module: syntax::Module<Resolved>) -> Result<Rc<ModuleInstance>> {
        let mut module_instance = ModuleInstance {
            types: module
                .types
                .into_iter()
                .map(|t| t.functiontype.into())
                .collect(),
            ..ModuleInstance::default()
        };

        // (Alloc 2.) Allocate functions
        // https://webassembly.github.io/spec/core/exec/modules.html#functions
        // We hold onto these so we can update the module instance at the end.
        let func_insts: Vec<Rc<FunctionInstance>> = module
            .funcs
            .into_iter()
            .map(|f| Self::instantiate_function(f, &module_instance))
            .map(Rc::new)
            .collect();
        let (func_count, func_offset) = self.store.alloc_funcs(func_insts.iter().cloned());
        module_instance.func_count = func_count;
        module_instance.func_offset = func_offset;
        println!("LOADED FUNCTIONS {} {}", func_count, func_offset);

        let mem_insts = module.memories.into_iter().map(MemInstance::new_ast);
        let (count, offset) = self.store.alloc_mems(mem_insts);
        module_instance.mem_count = count;
        module_instance.mem_offset = offset;

        // (Instantiation 5-10.) Generate global and elem init values
        // (Instantiation 5.) Create the module instance for global initialization
        let init_module_instance = Rc::new(module_instance.copy_for_init());

        // (Instantiation 6-7.) Create a frame with the instance, push it.
        self.stack.push_dummy_activation(init_module_instance)?;

        // (Instantiation 8.) Get global init vals and allocate globals.
        let global_insts: Vec<GlobalInstance> = module
            .globals
            .iter()
            .map(|g| {
                println!("COMPILE GLOBAL INIT EXPR {:x?}", g.init);
                let mut initexpr: Vec<u8> = Vec::new();
                initexpr.emit_expr(&g.init);
                let val = self.eval_expr(&initexpr).wrap("initializing global")?;
                Ok(GlobalInstance {
                    typ: g.globaltype.valtype,
                    val,
                })
            })
            .collect::<Result<_>>()?;
        let (count, offset) = self.store.alloc_globals(global_insts.into_iter());
        module_instance.global_count = count;
        module_instance.global_offset = offset;

        module_instance.exports = module
            .exports
            .into_iter()
            .map(|e| compile_export(e, &module_instance))
            .collect();

        self.stack.pop_activation()?;

        let rcinst = Rc::new(module_instance);

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
        let label = Label {
            arity: funcinst.functype.result.len() as u32,
            continuation: funcinst.body.len() as u32,
        };

        self.stack.push_label(label)?;

        self.enter(&funcinst.body)?;

        // NOTE: The compiled function has an `end` instruction at the end
        // which takes care of popping the label.

        // Due to validation, this should be equal to the frame above.
        self.stack.pop_activation()?;

        println!(
            "REMOVE FRAME {} {} {}",
            funcinst.locals.len(),
            funcinst.functype.params.len(),
            funcinst.functype.result.len(),
        );

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
                addr: ExternalVal::Func(addr),
            }) => Ok(addr),
            _ => err!("Method not found in module: {}", name),
        }?;

        // 1. Assert S.funcaddr exists
        // 2. Let funcinst = S.funcs[funcaddr]
        let funcinst = self
            .store
            .func(*funcaddr)
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
        self.invoke(*funcaddr)?;

        let mut results: Vec<Value> = vec![];
        for i in 0..funcinst.functype.result.len() {
            let result = self
                .stack
                .pop_value()
                .wrap(&format!("popping result {} for {}", i, name))?;
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
