use super::instance::{ElemInstance, FunctionInstance, GlobalInstance, MemInstance, TableInstance};
use super::values::Value;
use crate::{error, error::Result, logger::PrintLogger};
use std::iter::Iterator;
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
    logger: PrintLogger,
    pub funcs: Vec<Rc<FunctionInstance>>,
    pub tables: Vec<TableInstance>,
    pub mems: Vec<MemInstance>,
    pub globals: Vec<GlobalInstance>,
    pub elems: Vec<ElemInstance>,
}

impl Store {
    pub fn func(&self, addr: addr::FuncAddr) -> Result<Rc<FunctionInstance>> {
        self.funcs
            .get(addr as usize)
            .cloned()
            .ok_or_else(|| error!("no function at addr {}", addr))
    }

    pub fn global(&self, addr: addr::GlobalAddr) -> Result<Value> {
        self.globals
            .get(addr as usize)
            .ok_or_else(|| error!("no global at {}", addr))
            .map(|g| g.val)
    }

    pub fn set_global(&mut self, addr: addr::GlobalAddr, val: Value) -> Result<()> {
        let g = self
            .globals
            .get_mut(addr as usize)
            .ok_or_else(|| error!("no global at {}", addr))?;

        g.val = val;
        Ok(())
    }

    pub fn mem(&mut self, addr: addr::MemoryAddr) -> Result<&mut MemInstance> {
        self.mems
            .get_mut(addr as usize)
            .ok_or_else(|| error!("no mem at {}", addr))
    }

    pub fn table_mut(&mut self, addr: addr::TableAddr) -> Result<&mut TableInstance> {
        self.tables
            .get_mut(addr as usize)
            .ok_or_else(|| error!("no table at {}", addr))
    }

    pub fn grow_mem(&mut self, addr: addr::MemoryAddr, pgs: u32) -> Result<Option<u32>> {
        let mem = self.mem(addr)?;
        Ok(mem.grow(pgs))
    }

    // Use by the table.set and table.init ops
    pub fn copy_elems_to_table(
        &mut self,
        tabaddr: addr::TableAddr,
        elemaddr: addr::ElemAddr,
        src: usize,
        dst: usize,
        count: usize,
    ) -> Result<()> {
        let elems = &self
            .elems
            .get(elemaddr as usize)
            .ok_or_else(|| error!("no elem at {}", elemaddr))?
            .elems
            .get(src..src + count)
            .ok_or_else(|| error!("{} count={} out of bounds for {}", src, count, elemaddr))?;

        let table = &mut self
            .tables
            .get_mut(tabaddr as usize)
            .ok_or_else(|| error!("no table at {}", tabaddr))?
            .elem
            .get_mut(dst..dst + count)
            .ok_or_else(|| error!("{} count={} out of bounds for {}", dst, count, tabaddr))?;

        table.copy_from_slice(elems);
        Ok(())
    }

    pub fn elem_drop(&mut self, elemaddr: addr::ElemAddr) -> Result<()> {
        let elem = self
            .elems
            .get_mut(elemaddr as usize)
            .ok_or_else(|| error!("no elem at {}", elemaddr))?;

        elem.elems = Box::new([]);
        Ok(())
    }

    // Allocate a collection of functions.
    // Functions will be allocated in a contiguous block.
    // Returns the value of the first allocated fuction.
    pub fn alloc_funcs<I>(&mut self, funcs: I) -> std::ops::Range<addr::FuncAddr>
    where
        I: Iterator<Item = Rc<FunctionInstance>>,
    {
        let base_addr = self.funcs.len() as u32;
        self.funcs.extend(funcs);
        let count = self.funcs.len() as u32 - base_addr;
        base_addr..base_addr + count
    }

    // Allocate a collection of tables.
    // Tables will be allocated in a contiguous block.
    // Returns the value of the first allocated tables.
    pub fn alloc_tables<I>(&mut self, tables: I) -> std::ops::Range<addr::TableAddr>
    where
        I: Iterator<Item = TableInstance>,
    {
        let base_addr = self.tables.len() as u32;
        self.tables.extend(tables);
        let count = self.tables.len() as u32 - base_addr;
        base_addr..base_addr + count
    }

    // Allocate a collection of mems.
    // Mems will be allocated in a contiguous block.
    // Returns the value of the first allocated mems.
    pub fn alloc_mems<I>(&mut self, mems: I) -> std::ops::Range<addr::MemoryAddr>
    where
        I: Iterator<Item = MemInstance>,
    {
        let base_addr = self.mems.len() as u32;
        self.mems.extend(mems);
        let count = self.mems.len() as u32 - base_addr;
        base_addr..base_addr + count
    }

    pub fn alloc_globals<I>(&mut self, globals: I) -> std::ops::Range<addr::GlobalAddr>
    where
        I: Iterator<Item = GlobalInstance>,
    {
        let base_addr = self.globals.len() as u32;
        self.globals.extend(globals);
        let count = self.globals.len() as u32 - base_addr;
        base_addr..base_addr + count
    }

    pub fn alloc_elems<I>(&mut self, elems: I) -> std::ops::Range<addr::ElemAddr>
    where
        I: Iterator<Item = ElemInstance>,
    {
        let base_addr = self.elems.len() as u32;
        self.elems.extend(elems);
        let count = self.elems.len() as u32 - base_addr;
        base_addr..base_addr + count
    }
}
