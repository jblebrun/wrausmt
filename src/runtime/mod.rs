pub mod function;
pub mod store;

use store::Store; 
use super::module::Module;

#[derive(Debug)]
/// Contains all of the runtime state for the WASM interpreter.
pub struct Runtime<'lt> {
    /// The Store of the runtime, as described by the spec.
    store: Store<'lt>,
}

impl<'lt> Runtime<'lt> {
    pub fn new() -> Runtime<'lt> {
        Runtime {
            store: Store::new()
        }
    }

    pub fn load(&mut self, module: &'lt Module) {
        self.store.load(module);
    }
}
