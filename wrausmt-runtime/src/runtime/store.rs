use {
    super::{
        error::{Result, TrapKind},
        instance::{
            addr,
            addr::{Address, AddressRange, Addressable},
            DataInstance, ElemInstance, FunctionInstance, GlobalInstance, MemInstance,
            TableInstance,
        },
        values::{Ref, Value},
    },
    crate::impl_bug,
    std::{iter::Iterator, rc::Rc},
    wrausmt_common::true_or::TrueOr,
};

/// The WebAssembly Store as described in [the specification][Spec].
///
/// The store represents all global state that can be manipulated by WebAssembly
/// programs. It consists of the runtime representation of all instances of
/// functions, tables, memories, and globals, element segments, and data
/// segments that have been allocated during the life time of the abstract
/// machine. 1
///
/// It is an invariant of the semantics that no element or data instance is
/// addressed from anywhere else but the owning module instances.
/// Syntactically, the store is defined as a record listing the existing
/// instances of each category
///
/// * [FunctionInstance]
/// * [TableInstance]
/// * [MemInstance]
/// * [GlobalInstance]
/// * [ElemInstance]
/// * [DataInstance]
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#store
#[derive(Default, Debug)]
pub struct Store {
    // Functions need to be refcounted, because they can be recursively referenced.
    // (A function can eventually lead to code that calls it again).
    pub funcs:   Vec<Rc<FunctionInstance>>,
    pub tables:  Vec<TableInstance>,
    pub mems:    Vec<MemInstance>,
    pub globals: Vec<GlobalInstance>,
    pub elems:   Vec<ElemInstance>,
    pub datas:   Vec<DataInstance>,
}

impl Store {
    pub fn func(&self, addr: Address<addr::Function>) -> Result<Rc<FunctionInstance>> {
        Ok(self
            .funcs
            .get(addr.0 as usize)
            .cloned()
            .ok_or_else(|| impl_bug!("no function at addr {addr:?}"))?)
    }

    pub fn global(&self, addr: Address<addr::Global>) -> Result<Value> {
        Ok(self
            .globals
            .get(addr.0 as usize)
            .ok_or_else(|| impl_bug!("no global at addr {addr:?}"))?
            .val)
    }

    pub fn global_inst(&self, addr: Address<addr::Global>) -> Result<&GlobalInstance> {
        Ok(self
            .globals
            .get(addr.0 as usize)
            .ok_or_else(|| impl_bug!("no global at addr {addr:?}"))?)
    }

    pub fn set_global(&mut self, addr: Address<addr::Global>, val: Value) -> Result<()> {
        let g = self
            .globals
            .get_mut(addr.0 as usize)
            .ok_or_else(|| impl_bug!("no global at addr {addr:?}"))?;

        g.val = val;
        Ok(())
    }

    pub fn mem(&self, addr: Address<addr::Memory>) -> Result<&MemInstance> {
        Ok(self
            .mems
            .get(addr.0 as usize)
            .ok_or_else(|| impl_bug!("no mem at addr {addr:?}"))?)
    }

    pub fn mem_mut(&mut self, addr: Address<addr::Memory>) -> Result<&mut MemInstance> {
        Ok(self
            .mems
            .get_mut(addr.0 as usize)
            .ok_or_else(|| impl_bug!("no mem at addr {addr:?}"))?)
    }

    pub fn table(&self, addr: Address<addr::Table>) -> Result<&TableInstance> {
        Ok(self
            .tables
            .get(addr.0 as usize)
            .ok_or_else(|| impl_bug!("no table at addr {addr:?}"))?)
    }

    pub fn table_mut(&mut self, addr: Address<addr::Table>) -> Result<&mut TableInstance> {
        Ok(self
            .tables
            .get_mut(addr.0 as usize)
            .ok_or_else(|| impl_bug!("no table at addr {addr:?}"))?)
    }

    pub fn grow_mem(&mut self, addr: Address<addr::Memory>, pgs: u32) -> Result<Option<u32>> {
        let mem = self.mem_mut(addr)?;
        let old_size = mem.grow(pgs);
        if old_size.is_some() {
            mem.memtype.limits.lower = mem.size() as u32;
        }
        Ok(old_size)
    }

    pub fn grow_table(
        &mut self,
        addr: Address<addr::Table>,
        elems: u32,
        val: Ref,
    ) -> Result<Option<u32>> {
        let table = self.table_mut(addr)?;
        let growres = table.grow(elems, val);
        Ok(growres)
    }

    pub fn fill_mem(
        &mut self,
        addr: Address<addr::Memory>,
        n: usize,
        val: u8,
        i: usize,
    ) -> Result<()> {
        self.mem_mut(addr)?
            .data
            .get_mut(i..i + n)
            .ok_or(TrapKind::OutOfBoundsMemoryAccess(i, n))?
            .fill(val);
        Ok(())
    }

    // Use by the table.set and table.init ops
    pub fn copy_elems_to_table(
        &mut self,
        tabaddr: Address<addr::Table>,
        elemaddr: Address<addr::Elem>,
        src: usize,
        dst: usize,
        count: usize,
    ) -> Result<()> {
        let elems = &self
            .elems
            .get(elemaddr.0 as usize)
            .ok_or_else(|| impl_bug!("no elem at addr {elemaddr:?}"))?
            .elems
            .get(src..src + count)
            .ok_or(TrapKind::OutOfBoundsTableAccess(src, count))?;

        let table = &mut self
            .tables
            .get_mut(tabaddr.0 as usize)
            .ok_or_else(|| impl_bug!("no table at {tabaddr:?}"))?
            .elem
            .get_mut(dst..dst + count)
            .ok_or(TrapKind::OutOfBoundsTableAccess(src, count))?;

        table.copy_from_slice(elems);
        Ok(())
    }

    // Use by the table.set and table.init ops
    pub fn copy_table_to_table(
        &mut self,
        dstaddr: Address<addr::Table>,
        srcaddr: Address<addr::Table>,
        dst: usize,
        src: usize,
        count: usize,
    ) -> Result<()> {
        if dstaddr.0 == srcaddr.0 {
            let tbl = self.tables.get_mut(srcaddr.0 as usize).unwrap();
            (src + count <= tbl.elem.len())
                .true_or(TrapKind::OutOfBoundsTableAccess(src, count))?;
            (dst + count <= tbl.elem.len())
                .true_or(TrapKind::OutOfBoundsTableAccess(dst, count))?;
            tbl.elem.copy_within(src..src + count, dst);
        } else {
            let [src_table, dst_table] = self
                .tables
                .get_many_mut([srcaddr.0 as usize, dstaddr.0 as usize])
                .map_err(|_| impl_bug!("Couldn't get both tables {dstaddr:?} {srcaddr:?}"))?;

            let srcitems = src_table
                .elem
                .get_mut(src..src + count)
                .ok_or(TrapKind::OutOfBoundsTableAccess(src, count))?;

            let dstitems = dst_table
                .elem
                .get_mut(dst..dst + count)
                .ok_or(TrapKind::OutOfBoundsTableAccess(src, count))?;
            dstitems.copy_from_slice(srcitems);
        };
        Ok(())
    }

    pub fn copy_data_to_mem(
        &mut self,
        memaddr: Address<addr::Memory>,
        dataaddr: Address<addr::Data>,
        src: usize,
        dst: usize,
        count: usize,
    ) -> Result<()> {
        let data = self
            .datas
            .get(dataaddr.0 as usize)
            .ok_or_else(|| impl_bug!("no data at {dataaddr:?}"))?
            .bytes
            .get(src..src + count)
            .ok_or(TrapKind::OutOfBoundsMemoryAccess(src, count))?;

        self.mems
            .get_mut(memaddr.0 as usize)
            .ok_or_else(|| impl_bug!("no mem at {memaddr:?}"))?
            .write(0, dst, data)
    }

    pub fn copy_mem_to_mem(
        &mut self,
        memaddr: Address<addr::Memory>,
        src: usize,
        dst: usize,
        count: usize,
    ) -> Result<()> {
        let mem = self.mem_mut(memaddr)?;
        mem.copy_within(src, dst, count)
    }

    pub fn fill_table(
        &mut self,
        addr: Address<addr::Table>,
        n: usize,
        val: Ref,
        i: usize,
    ) -> Result<()> {
        self.table_mut(addr)?.fill(n, val, i)
    }

    pub fn elem_drop(&mut self, elemaddr: Address<addr::Elem>) -> Result<()> {
        let elem = self
            .elems
            .get_mut(elemaddr.0 as usize)
            .ok_or_else(|| impl_bug!("no elem at {elemaddr:?}"))?;

        elem.elems = Box::new([]);
        Ok(())
    }

    pub fn data_drop(&mut self, dataaddr: Address<addr::Data>) -> Result<()> {
        let data = self
            .datas
            .get_mut(dataaddr.0 as usize)
            .ok_or_else(|| impl_bug!("no elem at {dataaddr:?}"))?;

        data.bytes = Box::new([]);
        Ok(())
    }

    pub fn alloc<
        T: Addressable,
        StoredT,
        F: Fn(&mut Self) -> &mut Vec<StoredT>,
        OF: Fn(T) -> StoredT,
    >(
        &mut self,
        dest: F,
        items: impl Iterator<Item = Result<T>>,
        xform: OF,
    ) -> Result<AddressRange<T::AddressType>> {
        let dest = dest(self);
        let base_addr = dest.len() as u32;
        for item in items {
            dest.push(xform(item?));
        }
        let count = dest.len() as u32 - base_addr;
        Ok(AddressRange::new(base_addr, base_addr + count))
    }
}
