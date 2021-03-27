use super::super::module::*;
use super::function::FunctionInstance;

/// The WASM Store as described in the specification.
#[derive(Debug)]
pub struct Store<'lt> {
    funcs: Vec<FunctionInstance<'lt>>
}

impl<'lt> Store<'lt> {
    pub fn new() -> Store<'lt> {
        Store {
            funcs: vec![]
        }
    }

    pub fn load(&mut self, module: &'lt Module) {
        self.funcs.push(module.function_instance(0))
    }
}

impl Module {
    fn function_instance<'lt>(&'lt self, index: usize) -> FunctionInstance<'lt> {
        return FunctionInstance {
            functype: &self.types[self.funcs[index].functype as usize],
            code: &self.funcs[index]
        }

    }
}


