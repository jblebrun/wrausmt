use crate::error::{Result, ResultFrom};
use crate::module::Function;
use crate::runtime::error::ArgumentCountError;
use crate::runtime::module_instance::ModuleInstance;
use crate::runtime::Value;
use crate::types::FunctionType;
use std::rc::Rc;

/// A function entry in the store.
#[derive(Debug)]
pub struct FunctionInstance {
    /// The module instance that generated this function instance.
    pub module_instance: Rc<ModuleInstance>,

    /// The list of instructions in the function.
    pub code: Function,
}

impl FunctionInstance {
    pub fn functype(&self) -> &FunctionType {
        &self.module_instance.types[self.code.functype as usize]
    }

    pub fn validate_args(&self, args: &[Value]) -> Result<()> {
        let params_arity = self.functype().params.len();
        if params_arity != args.len() {
            return Err(ArgumentCountError::new(params_arity, args.len())).wrap("");
        }
        Ok(())
    }
}
