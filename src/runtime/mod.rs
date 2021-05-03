mod compile;
pub mod error;
pub mod exec;
pub mod instance;
pub mod instantiate;
pub mod stack;
pub mod store;
pub mod values;

use crate::{
    err, error as mkerror,
    error::{Result, ResultFrom},
    logger::{Logger, PrintLogger},
};
use std::{collections::HashMap, convert::TryInto, rc::Rc};

use {
    instance::{ExportInstance, ExternalVal, ModuleInstance},
    stack::Stack,
    store::addr,
    store::Store,
    values::Value,
};

#[derive(Debug)]
struct FunctionContext {
    body: Box<[u8]>,
    pc: usize,
    expr: bool,
}

#[derive(Debug, Default)]
struct CallStack {
    stack: Vec<FunctionContext>,
}

impl CallStack {
    fn top(&mut self) -> Result<&mut FunctionContext> {
        self.stack
            .last_mut()
            .ok_or_else(|| mkerror!("stack underflow"))
    }

    pub fn next_u32(&mut self) -> Result<u32> {
        let top = self.top()?;
        let result = u32::from_le_bytes(top.body[top.pc..top.pc + 4].try_into().wrap("idx")?);
        top.pc += 4;
        Ok(result)
    }

    pub fn next_u64(&mut self) -> Result<u64> {
        let top = self.top()?;
        let result = u64::from_le_bytes(top.body[top.pc..top.pc + 8].try_into().wrap("idx")?);
        top.pc += 8;
        Ok(result)
    }

    pub fn br(&mut self, cont: usize) -> Result<()> {
        let top = self.top()?;
        top.pc = cont;
        Ok(())
    }

    pub fn invoke(&mut self, body: Box<[u8]>) {
        println!("ENTER FUNCTION BODY {:x?}", body);
        self.stack.push(FunctionContext {
            body,
            pc: 0,
            expr: false,
        })
    }

    pub fn eval(&mut self, body: Box<[u8]>) {
        println!("ENTER FUNCTION BODY {:x?}", body);
        self.stack.push(FunctionContext {
            body,
            pc: 0,
            expr: true,
        })
    }

    pub fn ret(&mut self) -> Result<()> {
        self.stack
            .pop()
            .ok_or_else(|| mkerror!("stack underflow"))?;
        Ok(())
    }

    pub fn next_op(&mut self) -> Result<Option<u8>> {
        match self.stack.last_mut() {
            Some(top) => {
                if top.pc >= top.body.len() {
                    // End of function, implicit return
                    if top.expr {
                        return Ok(None);
                    } else {
                        println!("IMPLICIT RETURN");
                        return Ok(Some(0x0F));
                    }
                }
                let result = Ok(Some(top.body[top.pc]));
                top.pc += 1;
                result
            }
            None => Ok(None),
        }
    }
}

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

    callstack: CallStack,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime::default()
    }

    pub fn register<S: Into<String>>(&mut self, modname: S, module: Rc<ModuleInstance>) {
        self.registered.insert(modname.into(), module);
    }

    fn extend_addr_vec(vec: &mut Vec<u32>, range: std::ops::Range<u32>) {
        for i in range {
            vec.push(i);
        }
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

        self.callstack.invoke(funcinst.body.clone());

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

        self.run()?;

        println!("POPPING VALUES {}", funcinst.functype.result.len());
        let mut results: Vec<Value> = vec![];
        for i in 0..funcinst.functype.result.len() {
            let result = self
                .stack
                .pop_value()
                .wrap(&format!("popping result {} for {}", i, name))?;
            println!("POPPED {:?}", result);
            results.push(result);
        }

        // pop the dummy frame
        // due to validation, this will be the one we pushed above.
        println!("POP HOST DUMMY");
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
