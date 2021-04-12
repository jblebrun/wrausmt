pub mod error;
mod exec;
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

use self::instance::FunctionInstance;
use {
    instance::{ExportInstance, ExternalVal, ModuleInstance},
    stack::{ActivationFrame, Label, Stack},
    store::addr,
    store::Store,
    values::Value,
};

#[derive(Debug, Default)]
/// Contains all of the runtime state for the WASM interpreter.
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

    pub fn load(&mut self, module: Module) -> Rc<ModuleInstance> {
        let mut module_instance = ModuleInstance {
            types: module.types,
            ..ModuleInstance::default()
        };

        let func_insts: Vec<FunctionInstance> = module.funcs.into_vec().drain(..)
            .map(|f| FunctionInstance::new(f, &module_instance.types))
            .collect();

        module_instance.func_count = func_insts.len();
        module_instance.func_offset = self.store.alloc_funcs(func_insts);

        let exports = module.exports.into_vec().drain(..)
            .map(|e| ExportInstance::new(e, &module_instance))
            .collect();

        module_instance.exports = exports;

        let rcinst = Rc::new(module_instance);

        self.store.update_func_module_instance(&rcinst);

        rcinst
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
        let param_count = funcinst.functype.params.len();
        let mut vals: Vec<Value> = vec![];
        for _ in 0..param_count {
            vals.push(self.stack.pop_value()?);
        }

        // 8. Let val0* be the list of zero values (other locals). TODO
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

        self.stack.push_label(Label {
            arity: funcinst.functype.result.len() as u32,
            continuation: 0,
        });

        self.enter(&funcinst.code.body)
    }

    /// Invocation of a function by the host.
    pub fn call(
        &mut self,
        mod_instance: Rc<ModuleInstance>,
        name: &str,
        vals: &[Value],
    ) -> Result<Value> {
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

        // assume single result for now
        let result = self.stack.pop_value()?;

        // pop the label
        self.stack.pop_label()?;

        // pop the frame
        self.stack.pop_activation()?;

        Ok(result)
    }
}
