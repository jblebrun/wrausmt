pub mod function;
pub mod store;

use store::Store; 
use super::module::Module;

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

    pub fn load(&mut self, module: Module) {
        self.store.load(module);
    }
}
