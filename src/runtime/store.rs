use super::super::module::*;
use super::function::FunctionInstance;
use super::super::types::FunctionType;

/// The WASM Store as described in the specification.
#[derive(Debug)]
pub struct Store {
    funcs: Vec<FunctionInstance>
}

impl Store {
    pub fn new() -> Store {
        Store {
            funcs: vec![]
        }
    }

    /// Load the provided module into this store.
    /// THe module will be owned by the store after this call.
    pub fn load(&mut self, module: Module) {
        let types = &module.types;
        module.funcs.into_vec()
            .drain(..)
            .for_each(|f| {
            self.funcs.push(FunctionInstance {
                functype: types[f.functype as usize].clone(),
                code: f
            });
        });
    }
}

