use crate::{error::{Result, ResultFrom}, instructions::Expr, types::ValueType};
use crate::module::Function;
use crate::runtime::error::ArgumentCountError;
use crate::runtime::instance::ModuleInstance;
use crate::runtime::Value;
use crate::types::FunctionType;
use std::rc::Rc;
use std::cell::RefCell;
use crate::error;

/// A function instance is the runtime representation of a function. [Spec][Spec]
///
/// It effectively is a closure of the original function over the runtime module
/// instance of its originating module. The module instance is used to resolve
/// references to other definitions during execution of the function.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#function-instances
#[derive(Debug)]
pub struct FunctionInstance {
    pub functype: FunctionType,
    pub module_instance: RefCell<Option<Rc<ModuleInstance>>>,
    
    /// The locals declare a vector of mutable local variables and their types. These variables are
    /// referenced through local indices in the function's body. The index of the first local is
    /// the smallest index not referencing a parameter.
    pub locals: Box<[ValueType]>,

    /// The body is an instruction sequence that upon termination must produce a stack matching the
    /// function type's result type.
    pub body: Box<Expr>,
}

/// A host function is a function expressed outside WebAssembly but passed to a
/// module as an import. The definition and behavior of host functions are
/// outside the scope of this specification. For the purpose of this
/// specification, it is assumed that when invoked, a host function behaves
/// non-deterministically, but within certain constraints that ensure the
/// integrity of the runtime.
///
/// Note: Host functions are not yet used in this implementation.
#[allow(dead_code)]
struct HostFunc {
    functype: FunctionType,
    //hostfunc: HostFunc,
}

impl FunctionInstance {
    pub fn new(func: Function, types: &[FunctionType]) -> FunctionInstance {
        FunctionInstance {
            functype: types[func.functype as usize].clone(),
            module_instance: RefCell::new(None),
            locals: func.locals,
            body: func.body
        }
    }

    pub fn validate_args(&self, args: &[Value]) -> Result<()> {
        let params_arity = self.functype.params.len();
        if params_arity != args.len() {
            return Err(ArgumentCountError::new(params_arity, args.len())).wrap("");
        }
        Ok(())
    }

    pub fn module_instance(&self) -> Result<Rc<ModuleInstance>> {
        self.module_instance.borrow().clone().ok_or_else(|| error!("no module instance on function"))
    }
}
