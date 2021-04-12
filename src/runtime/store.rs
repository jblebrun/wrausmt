use super::instance::{FunctionInstance, ModuleInstance};
use crate::{
    error,
    error::Result,
};
use std::rc::Rc;

/// Function instances, table instances, memory instances, and global instances,
/// element instances, and data instances in the store are referenced with
/// abstract addresses. These are simply indices into the respective store
/// component. In addition, an embedder may supply an uninterpreted set of host
/// addresses.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#addresses
pub mod addr {
    pub type FuncAddr = u32;
    pub type TableAddr = u32;
    pub type MemoryAddr = u32;
    pub type GlobalAddr = u32;
    pub type ElemAddr = u32;
    pub type DataAddr = u32;
    pub type ExternAddr = u32;
}

/// The WebAssembly Store as described in [the specification][Spec].
///
/// The store represents all global state that can be manipulated by WebAssembly
/// programs. It consists of the runtime representation of all instances of
/// functions, tables, memories, and globals, element segments, and data segments
/// that have been allocated during the life time of the abstract machine. 1
///
/// It is an invariant of the semantics that no element or data instance is
/// addressed from anywhere else but the owning module instances.
/// Syntactically, the store is defined as a record listing the existing
/// instances of each category
///
/// * [FunctionInstance](FunctionInstance)
/// * [TableInstance](super::instance::TableInstance)
/// * [MemInstance](super::instance::MemInstance)
/// * [GlobalInstance](super::instance::GlobalInstance)
/// * [ElemInstance](super::instance::ElemInstance)
/// * [DataInstance](super::instance::DataInstance)
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#store
#[derive(Default, Debug)]
pub struct Store {
    // FunctionInstance contain Functions, and thus the code
    // to run. They will be used by execution threads, so
    // are stored as Rc.
    pub funcs: Vec<Rc<FunctionInstance>>,
}

impl Store {
    pub fn new() -> Store {
        Store { funcs: vec![] }
    }

    pub fn func(&self, addr: addr::FuncAddr) -> Result<Rc<FunctionInstance>> {
        self.funcs
            .get(addr as usize)
            .cloned()
            .ok_or_else(|| error!("no function at addr {}", addr))
    }

    /// Update the [FunctionInstancs][FunctionInstance] in this store for the provided modules.
    pub fn update_func_module_instance(&mut self, module_instance: &Rc<ModuleInstance>) {
        let count = module_instance.func_count;
        let start = module_instance.func_offset as usize;
        let end = start + count;
        for i in start..end  {
            self.funcs[i].module_instance.replace(Some(module_instance.clone()));
        }
    }

    // Allocate a collection of functions.
    // Functions will be allocated in a contiguous block.
    // Returns the value of the first allocated fuction.
    pub fn alloc_funcs(&mut self, funcs: Vec<FunctionInstance>) -> addr::FuncAddr {
        let base_addr = self.funcs.len();
        for f in funcs {
            self.funcs.push(Rc::new(f));
        }
        base_addr as addr::FuncAddr
    }
}
