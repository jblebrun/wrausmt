pub mod function;
pub mod store;
pub mod stack;
mod exec;


use store::Store; 
use stack::Stack;
use super::module::Module;
use store::ModuleInstance;
use std::rc::Rc;
use stack::StackEntry;
use stack::Frame;
use super::instructions::Inst;

#[derive(Debug)]
/// Contains all of the runtime state for the WASM interpreter.
pub struct Runtime {
    /// The Store of the runtime, as described by the spec.
    store: Store,

    stack: Stack,

    // TODO next: Current rame management
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            store: Store::new(),
            stack: Stack::new()
        }
    }

    pub fn load(&mut self, module: Module) -> Rc<ModuleInstance> {
        self.store.load(module)
    }

    pub fn execute(&mut self, block: &[Inst]) {
        for inst in block {
            inst.execute(self);
        }
    }

    pub fn call<'lt>(
        &mut self, 
        mod_instance: Rc<ModuleInstance>, 
        name: &str,
        arg: u64
    ) {
        let found = mod_instance.resolve(name); 
        match found {
            None => {
                println!("Not found!");
                return
            },
            Some(export) => {
                println!("Found it! {:?}", export);

                let func = self.store.funcs[export.addr as usize].clone();

                // create activation frame
                self.stack.push( StackEntry::Activation { 
                    arity: func.result_arity() as u32,
                    frame: Frame {
                        arity: func.params_arity() as u32,
                        locals: Box::new([arg]),
                        module: mod_instance.clone()
                    }
                });

                // TODO - mark current frame

                // create label
                self.stack.push ( StackEntry::Label {
                    arity: func.result_arity() as u32,
                    continuation: Rc::new([])
                });

                // start executing
                let body = &func.code.body;

                self.execute(body);
                
                println!("{:?}", self.stack.pop());
                // retrieve result from stack
            }
        }
    }
}
