use super::*;
use crate::{runtime::store::addr, types::FunctionType};

/// A module instance is the runtime representation of a module. [Spec][Spec]
///
/// It is created by instantiating a module, and collects runtime representations
/// of all entities that are imported, defined, or exported by the module.
///
/// Each component references runtime instances corresponding to respective
/// declarations from the original module – whether imported or defined – in the
/// order of their static indices. Function instances, table instances, memory
/// instances, and global instances are referenced with an indirection through
/// their respective addresses in the store. [Spec][Spec]
///
/// It is an invariant of the semantics that all export instances in a given
/// module instance have different names.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#module-instances
#[derive(Debug, Default, Clone)]
pub struct ModuleInstance {
    types: Box<[FunctionType]>,
    exports: Box<[ExportInstance]>,
    funcs: Box<[addr::FuncAddr]>,
    tables: Box<[addr::TableAddr]>,
    mems: Box<[addr::MemoryAddr]>,
    globals: Box<[addr::GlobalAddr]>,
    elems: Box<[addr::ElemAddr]>,
    data: Box<[addr::DataAddr]>,
}

impl ModuleInstance {
    pub fn func(&self, idx: u32) -> addr::FuncAddr {
        self.funcs[idx as usize]
    }

    pub fn table(&self, idx: u32) -> addr::TableAddr {
        self.tables[idx as usize]
    }

    pub fn mem(&self, idx: u32) -> addr::MemoryAddr {
        self.mems[idx as usize]
    }

    pub fn global(&self, idx: u32) -> addr::GlobalAddr {
        self.globals[idx as usize]
    }

    pub fn elem(&self, idx: u32) -> addr::ElemAddr {
        println!("GET ELEM {} FROM {:?}", idx, self.elems);
        self.elems[idx as usize]
    }

    pub fn data(&self, idx: u32) -> addr::DataAddr {
        self.data[idx as usize]
    }

    pub fn resolve(&self, name: &str) -> Option<&ExportInstance> {
        let found = self.exports.iter().find(|e| e.name == name);

        found
    }
}

#[derive(Debug, Default, Clone)]
pub struct ModuleInstanceBuilder {
    pub types: Vec<FunctionType>,
    pub exports: Vec<ExportInstance>,
    pub funcs: Vec<addr::FuncAddr>,
    pub tables: Vec<addr::TableAddr>,
    pub mems: Vec<addr::MemoryAddr>,
    pub globals: Vec<addr::GlobalAddr>,
    pub elems: Vec<addr::ElemAddr>,
    pub data: Vec<addr::DataAddr>,
}

impl ModuleInstanceBuilder {
    pub fn add_external_val(&mut self, ev: ExternalVal) {
        match ev {
            ExternalVal::Func(addr) => self.funcs.push(addr),
            ExternalVal::Table(addr) => self.tables.push(addr),
            ExternalVal::Memory(addr) => self.mems.push(addr),
            ExternalVal::Global(addr) => self.globals.push(addr),
        }
    }
    pub fn build(self) -> ModuleInstance {
        ModuleInstance {
            types: self.types.into_boxed_slice(),
            exports: self.exports.into_boxed_slice(),
            funcs: self.funcs.into_boxed_slice(),
            tables: self.tables.into_boxed_slice(),
            mems: self.mems.into_boxed_slice(),
            globals: self.globals.into_boxed_slice(),
            elems: self.elems.into_boxed_slice(),
            data: self.data.into_boxed_slice(),
        }
    }
}
