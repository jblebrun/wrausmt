use super::instance::{FunctionInstance, MemInstance, TableInstance};
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
    pub funcs: Vec<Rc<FunctionInstance>>,
    pub tables: Vec<Rc<TableInstance>>,
    pub mems: Vec<Rc<MemInstance>>,
}

impl Store {
    pub fn new() -> Store {
        Self::default()
    }

    pub fn func(&self, addr: addr::FuncAddr) -> Result<Rc<FunctionInstance>> {
        self.funcs
            .get(addr as usize)
            .cloned()
            .ok_or_else(|| error!("no function at addr {}", addr))
    }

    // Allocate a collection of functions.
    // Functions will be allocated in a contiguous block.
    // Returns the value of the first allocated fuction.
    pub fn alloc_funcs<I>(&mut self, funcs: I) -> (usize, addr::FuncAddr)
        where I : Iterator<Item=Rc<FunctionInstance>>
    {
        let base_addr = self.funcs.len();
        self.funcs.extend(funcs);
        let count = self.funcs.len()-base_addr;
        (count, base_addr as addr::FuncAddr)
    }

    // Allocate a collection of tables.
    // Tables will be allocated in a contiguous block.
    // Returns the value of the first allocated tables.
    pub fn alloc_tables<I>(&mut self, tables: I) -> (usize, addr::TableAddr) 
        where I : Iterator<Item=TableInstance>
    {
        let base_addr = self.tables.len();
        self.tables.extend(tables.map(Rc::new));
        let count = self.tables.len()-base_addr;
        (count, base_addr as addr::TableAddr)
    }

    // Allocate a collection of mems.
    // Mems will be allocated in a contiguous block.
    // Returns the value of the first allocated mems.
    pub fn alloc_mems<I>(&mut self, mems: I) -> (usize, addr::MemoryAddr) 
        where I : Iterator<Item=MemInstance>
    {
        let base_addr = self.mems.len();
        self.mems.extend(mems.map(Rc::new));
        let count = self.mems.len()-base_addr;
        (count, base_addr as addr::MemoryAddr)
    }
}
