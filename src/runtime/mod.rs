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
                panic!("Method not found: {}", name);
            },
            Some(export) => {
                let func = self.store.funcs[export.addr as usize].clone();

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
                let body = &func.code.body;

                self.execute(body);
                
                println!("{:?}", self.stack.pop());

                // pop the label
                self.stack.pop();

                // pop the frame
                self.stack.pop();

                // clear current frame
                self.current_frame = None;
            }
        }
    }
}
