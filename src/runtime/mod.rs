pub mod function;
pub mod store;
pub mod stack;
mod exec;


use super::module::Module;
use std::rc::Rc;
use stack::{Stack, StackEntry, Frame};
use store::{Export, ExternalVal, ModuleInstance, Store};
use super::error::{Error, Result};

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

    pub fn call<'lt>(
        &mut self, 
        mod_instance: Rc<ModuleInstance>, 
        name: &str,
        arg: u64
    ) -> Result<u64> {
        let found = mod_instance.resolve(name); 
        match found {
            Some(Export { name: _, addr: ExternalVal::Func(addr) }) => {
                let func = self.store.funcs[*addr as usize].clone();

                let frame = Rc::new(Frame {
                    locals: Box::new([arg]),
                    module: mod_instance.clone()
                });
                
                // create activation frame
                self.stack.push( StackEntry::Activation { 
                    arity: func.result_arity() as u32,
                    frame: frame.clone()
                });

                self.current_frame = Some(frame);

                // create label
                self.stack.push ( StackEntry::Label {
                    arity: func.result_arity() as u32,
                    continuation: Rc::new([])
                });

                // start executing
                self.invoke(&func.code.body)?;
                
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
                    _ => Err(Error::new(format!("Bad stack type {:?}", result)))
                }
            },
            _ => Err(Error::new(format!("Method not found: {}", name)))
        }
    }
}
