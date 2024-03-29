use {
    self::instance::addr::{self, Address},
    crate::log_tag::Tag,
    wrausmt_common::logger::{Logger, PrintLogger},
};

pub mod error;
pub mod exec;
pub mod instance;
pub mod instantiate;
pub mod stack;
pub mod store;
pub mod values;

use {
    self::instance::FunctionInstance,
    crate::{impl_bug, runtime::error::RuntimeErrorKind},
    error::Result,
    instance::{ExportInstance, ExternalVal, ModuleInstance},
    stack::Stack,
    std::{collections::HashMap, rc::Rc},
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

    pub fn register(&mut self, modname: impl Into<String>, module: Rc<ModuleInstance>) {
        self.registered.insert(modname.into(), module);
    }

    pub fn invoke_addr(&mut self, addr: Address<addr::Function>) -> Result<()> {
        // 1. Assert S.funcaddr exists
        // 2. Let funcinst = S.funcs[funcaddr]
        let funcinst = self.store.func(addr)?;
        self.invoke(funcinst)
    }

    pub fn invoke(&mut self, funcinst: Rc<FunctionInstance>) -> Result<()> {
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

        self.logger.log(Tag::Activate, || {
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
            _ => Err(RuntimeErrorKind::MethodNotFound(name.to_owned())),
        }?;

        self.logger
            .log(Tag::Host, || format!("calling {} at {:?}", name, funcaddr));
        // 1. Assert S.funcaddr exists
        // 2. Let funcinst = S.funcs[funcaddr]
        let funcinst = self.store.func(funcaddr)?;

        // 3. Let [tn_1] -> [tm_2] be the function type.
        // 4. If the length of vals is different then the number of vals provided, fail.
        // 5. For each value type, if not matching declared type, fail.
        funcinst.validate_args(vals)?;

        // 6. Let F be a dummy frame. (Represents a dummy "caller" for the function to
        //    invoke).
        // 7. Push F to the stack.
        self.stack.push_dummy_activation(mod_instance.clone())?;

        // 8. Push the values to the stack.
        for val in vals {
            self.stack.push_value(*val);
        }

        // 9. Invoke the function.
        self.invoke_addr(funcaddr)?;

        let mut results: Vec<Value> = vec![];
        for _ in 0..funcinst.functype.result.len() {
            let result = self.stack.pop_value()?;

            self.logger
                .log(Tag::Host, || format!("POPPED HOST RESULT {:?}", result));
            results.push(result);
        }

        // pop the dummy frame
        // due to validation, this will be the one we pushed above.
        self.stack.pop_activation()?;

        // Since we don't do validation yet, do some checking here to make sure things
        // seem ok.
        if let Ok(v) = self.stack.pop_value() {
            Err(impl_bug!("values still on stack {:?}", v))?;
        }

        if let Ok(l) = self.stack.peek_label() {
            Err(impl_bug!("labels still on stack {:?}", l))?;
        }

        if self.stack.activation_depth() != 0 {
            Err(impl_bug!("frames still on stack"))?;
        }
        Ok(results)
    }

    pub fn get_global(&mut self, mod_instance: &Rc<ModuleInstance>, name: &str) -> Result<Value> {
        let globaladdr = match mod_instance.resolve(name) {
            Some(ExportInstance {
                name: _,
                addr: ExternalVal::Global(idx),
            }) => *idx,
            _ => Err(RuntimeErrorKind::MethodNotFound(name.to_owned()))?,
        };

        self.logger
            .log(Tag::Host, || format!("calling {name} at {globaladdr:?}"));
        // 1. Assert S.funcaddr exists
        // 2. Let funcinst = S.funcs[funcaddr]
        let globalinst = self.store.global(globaladdr)?;

        Ok(globalinst)
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
                $runtime.call(&$mod_inst, $cmd, &[$v1.into()])
            };
            ( $cmd:expr, $v1:expr, $v2:expr ) => {
                $runtime.call(&$mod_inst, $cmd, &[$v1.into(), $v2.into()])
            };
        }
    };
}
