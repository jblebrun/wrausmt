pub mod function;
pub mod store;
pub mod stack;
pub mod error;
pub mod values;
mod exec;

use std::rc::Rc;
use super::{
    module::Module,
    error::{ResultFrom, Result},
    err
};
use {
    values::Value,
    stack::{Stack, StackEntry, Frame},
    store::{Export, ExternalVal, ModuleInstance, Store, FuncAddr},
};

#[derive(Debug)]
/// Contains all of the runtime state for the WASM interpreter.
pub struct Runtime {
    /// The Store of the runtime, as described by the spec.
    store: Store,

    /// The runtime stack.
    stack: Stack,

    /// The topmost frame in the stack, if there is one.
    current_frame: Option<Rc<Frame>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            store: Store::new(),
            stack: Stack::new(),
            current_frame: None
        }
    }

    pub fn load(&mut self, module: Module) -> Rc<ModuleInstance> {
        self.store.load(module)
    }

    pub fn invoke(
        &mut self,
        addr: FuncAddr,
    ) -> Result<()> {
        // 1. Assert S.funcaddr exists
        // 2. Let funcinst = S.funcs[funcaddr]
        let funcinst = self.store.func(addr)?;
        
        // 3. Let [tn_1] -> [tm_2] be the function type.
        // 4. Let t* be the list of locals.
        // 5. Let instr* end be the code body
        // 6. Assert (due to validation) n values on the stack
        // 7. Pop val_n from the stack
        let param_count = funcinst.functype().params.len();
        let mut vals: Vec<Value> = vec![];
        for _ in 0..param_count {
            vals.push(self.stack.pop_value()?);
        }

        // 8. Let val0* be the list of zero values (other locals). TODO
        for localtype in funcinst.code.locals.iter() {
            vals.push(localtype.default());
        }

        // 9. Let F be the frame.
        let locals = vals.into_boxed_slice();
        let frame = Rc::new(Frame::new(&funcinst.module_instance, locals));

        // Impl detail: store ref to current frame.
        self.current_frame = Some(frame.clone());

        // 10. Push activation w/ arity m onto the stack.
        self.stack.push( StackEntry::Activation { 
            arity: funcinst.functype().result.len() as u32,
            frame 
        });

        // 11. Let L be the Label with continuation at function end.
        // 12. Enter the instruction sequence with the label.
        
        // Impl TODO: label-only stack for convenience?
        self.stack.push ( StackEntry::Label {
            arity: funcinst.functype().result.len() as u32,
            continuation: Rc::new([])
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
            Some(Export { name: _, addr: ExternalVal::Func(addr)}) => Ok(addr),
            _ => err!("Method not found in module: {}", name)
        }?;
        
        // 1. Assert S.funcaddr exists
        // 2. Let funcinst = S.funcs[funcaddr]
        let funcinst = self.store.func(*funcaddr).wrap(&format!("for name {}", name))?;

        // 3. Let [tn_1] -> [tm_2] be the function type.
        // 4. If the length of vals is different then the number of vals provided, fail.
        // 5. For each value type, if not matching declared type, fail.
        funcinst.validate_args(vals).wrap(&format!("for {}", name))?;

        // 6. Let F be a dummy frame. (Represents a dummy "caller" for the function to invoke).
        // 7. Push F to the stack. 
        let dummy_frame = Rc::new(Frame::dummy());
        self.stack.push(StackEntry::Activation { arity: 0, frame: dummy_frame});

        // 8. Push the values to the stack.
        for val in vals {
            self.stack.push(val.into());
        }
        
        // 9. Invoke the function.
        self.invoke(*funcaddr)?;
         
        // assume single result for now
        let result = self.stack.pop().unwrap();

        // pop the label
        self.stack.pop();

        // pop the frame
        self.stack.pop();

        // clear current frame
        self.current_frame = None;

        match result {
            StackEntry::Value(val) => Ok(val),
            _ => err!("Bad stack type {:?}", result)
        }
    }
}
