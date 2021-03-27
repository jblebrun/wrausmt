pub mod function;
pub mod store;

use store::Store; 
use super::module::Module;
use store::ModuleInstance;
use std::rc::Rc;

#[derive(Debug)]
/// Contains all of the runtime state for the WASM interpreter.
pub struct Runtime {
    /// The Store of the runtime, as described by the spec.
    store: Store,
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            store: Store::new()
        }
    }

    pub fn load(&mut self, module: Module) -> Rc<ModuleInstance> {
        self.store.load(module)
    }

    pub fn call(&self, mod_instance: &ModuleInstance, name: &str) {
        let found = mod_instance.resolve(name); 
        match found {
            None => {
                println!("Not found!");
                return
            },
            Some(_) => println!("Found it!")
            //invoke method
        }
    }
}
