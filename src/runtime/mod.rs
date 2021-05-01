pub mod error;
pub mod exec;
pub mod instance;
pub mod stack;
pub mod store;
pub mod values;

use crate::{
    err,
    error::{Result, ResultFrom},
    module::Module,
};
use std::rc::Rc;

use self::instance::GlobalInstance;

use {
    instance::{ExportInstance, ExternalVal, ElemInstance, FunctionInstance, MemInstance, ModuleInstance, TableInstance},
    stack::{ActivationFrame, Label, Stack},
    store::addr,
    store::Store,
    values::Value
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
    pub fn load(&mut self, module: Module) -> Result<Rc<ModuleInstance>> {
        // TODO Resolve imports
        for import in module.imports.iter() {
            println!("NEED TO RESOLVE {}:{}", import.module_name, import.name);
        }
        self.instantiate(module)
    }

    /// Instntiation (and allocation) of the provided module, roughly following the
    /// specification. Allocation and instantiation are described as two independent
    /// [Allocation](https://webassembly.github.io/spec/core/exec/modules.html#alloc-module)
    /// [Instantiation](https://webassembly.github.io/spec/core/exec/modules.html#instantiate-module)
    fn instantiate(&mut self, module: Module) -> Result<Rc<ModuleInstance>> {
        // (Instantiate 1-4.) TODO Validate module

        let mut module_instance = ModuleInstance {
            types: module.types,
            ..ModuleInstance::default()
        };

        // (Alloc 2.) Allocate functions
        // https://webassembly.github.io/spec/core/exec/modules.html#functions
        let new_func_inst = |f| Rc::new(FunctionInstance::new(f, &module_instance.types));
        // We hold onto these so we can update the module instance at the end.
        let func_insts: Vec<Rc<FunctionInstance>> = module.funcs
            .into_vec()
            .into_iter()
            .map(new_func_inst)
            .collect();
        let (func_count, func_offset) = self.store.alloc_funcs(func_insts.iter().cloned());
        module_instance.func_count = func_count;
        module_instance.func_offset = func_offset;

        // (Alloc 3.) Allocate tables
        let table_insts = module.tables.into_vec().into_iter().map(TableInstance::new);
        let (count, offset) = self.store.alloc_tables(table_insts);
        module_instance.table_count = count;
        module_instance.table_offset = offset;

        // (Alloc 4.) Allocate mem
        let mem_insts = module.mems.into_vec().into_iter().map(MemInstance::new);
        let (count, offset) = self.store.alloc_mems(mem_insts);
        module_instance.mem_count = count;
        module_instance.mem_offset = offset;

        // (Instantiation 5-10.) Generate global and elem init values
        // (Instantiation 5.) Create the module instance for global initialization
        let init_module_instance = Rc::new(module_instance.copy_for_init());
       
        // (Instantiation 6-7.) Create a frame with the instance, push it.
        let init_frame = ActivationFrame::new(0, init_module_instance, Box::new([]));
        self.stack.push_activation(init_frame);

        // (Instantiation 8.) Get global init vals and allocate globals.
        let global_insts: Vec<GlobalInstance> = module.globals
            .iter()
            .map(|g| {
                let val = self.eval_expr(&g.init)?;
                Ok(GlobalInstance { typ: g.typ.valtype, val })
            })
            .collect::<Result<_>>()?;
        let (count, offset) = self.store.alloc_globals(global_insts.into_iter());
        module_instance.global_count = count;
        module_instance.global_offset = offset;

        // (Instantian 9.) Get elem init vals
        let elem_insts: Vec<ElemInstance> = module.elems
            .iter()
            .map(|e| {
                let elems = e.init.iter()
                    .map(|ei| self.eval_ref_expr(&ei))
                    .collect::<Result<_>>()?;
                Ok(ElemInstance { elemtype: e.typ, elems })
            })
            .collect::<Result<_>>()?;
        let (count, offset) = self.store.alloc_elems(elem_insts.into_iter());
        module_instance.elem_count = count;
        module_instance.elem_offset = offset;
            
        self.stack.pop_activation()?;

        // (Alloc 5.) Globals

        // (Alloc 6.) Elements

        // (Alloc 7.) Data

        // (Alloc 18.) Allocate exports.
        let new_export_inst = |e| ExportInstance::new(e, &module_instance);
        let exports = module.exports
            .into_vec()
            .into_iter()
            .map(new_export_inst)
            .collect();

        // (Alloc 19) Collect exports
        module_instance.exports = exports;

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
        let mut vals: Vec<Value> = vec![];
        for _ in funcinst.functype.params.iter() {
            let v = self.stack.pop_value()?;
            println!("PUSH PARAM {} {:?}", vals.len(),  v);
            vals.push(v);
        }

        // TODO - should calls work by moving the stack pointer, rather than
        // by pushing/popping? This would simplify framing, too; activation frames
        // would no longer need locals.
        vals.reverse();

        // 8. Let val0* be the list of zero values (other locals). 
        for localtype in funcinst.code.locals.iter() {
            vals.push(localtype.default());
        }

        // 9. Let F be the frame.
        // 10. Push activation w/ arity m onto the stack.
        self.stack.push_activation(ActivationFrame::new(
            funcinst.functype.result.len() as u32,
            funcinst.module_instance()?,
            vals.into_boxed_slice(),
        ));

        // 11. Let L be the Label with continuation at function end.
        // 12. Enter the instruction sequence with the label.
        let label = Label {
            arity: funcinst.functype.result.len() as u32,
            continuation: funcinst.code.body.len() as u32,
        };
        
        self.stack.push_label(label);

        self.enter(&funcinst.code.body)?;
        
        // Due to validation, this shouould be equal to label.
        // TODO - validation.
        self.stack.pop_label()?;
        
        // Due to validation, this should be equal to the frame above.
        self.stack.pop_activation()?;

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
        self.stack.push_activation(ActivationFrame::default());

        // 8. Push the values to the stack.
        for val in vals {
            self.stack.push_value(*val);
        }

        // 9. Invoke the function.
        self.invoke(*funcaddr)?;

        let mut results: Vec<Value> = vec![];
        for i in 0..funcinst.functype.result.len() {
            let result = self.stack.pop_value()
                .wrap(&format!("popping result {} for {}", i, name))?;
            results.push(result);
        }

        // pop the dummy frame
        // due to validation, this will be the one we pushed above.
        self.stack.pop_activation()?;

        // Since we don't do validation yet, do some checking here to make sure things seem ok.
        if let Ok(v) = self.stack.pop_value() {
            return err!("values still on stack {:?}", v)
        }

        if let Ok(l) = self.stack.peek_label() {
            return err!("labels still on stack {:?}", l)
        }

        if let Ok(f) = self.stack.peek_activation() {
            return err!("frames still on stack {:?}", f)
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
            }
        }
    }
}
