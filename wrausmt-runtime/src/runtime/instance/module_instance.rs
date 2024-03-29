use {
    super::{addr::Address, *},
    crate::syntax::types::FunctionType,
};

/// A module instance is the runtime representation of a module. [Spec][Spec]
///
/// It is created by instantiating a module, and collects runtime
/// representations of all entities that are imported, defined, or exported by
/// the module.
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
    types:   Box<[FunctionType]>,
    exports: Box<[ExportInstance]>,
    funcs:   Box<[Address<addr::Function>]>,
    tables:  Box<[Address<addr::Table>]>,
    mems:    Box<[Address<addr::Memory>]>,
    globals: Box<[Address<addr::Global>]>,
    elems:   Box<[Address<addr::Elem>]>,
    data:    Box<[Address<addr::Data>]>,
}

impl ModuleInstance {
    pub fn func_type(&self, idx: u32) -> &FunctionType {
        &self.types[idx as usize]
    }

    pub fn func(&self, idx: u32) -> Address<addr::Function> {
        self.funcs[idx as usize]
    }

    pub fn table(&self, idx: u32) -> Address<addr::Table> {
        self.tables[idx as usize]
    }

    pub fn mem(&self, idx: u32) -> Address<addr::Memory> {
        self.mems[idx as usize]
    }

    pub fn global(&self, idx: u32) -> Address<addr::Global> {
        self.globals[idx as usize]
    }

    pub fn elem(&self, idx: u32) -> Address<addr::Elem> {
        self.elems[idx as usize]
    }

    pub fn data(&self, idx: u32) -> Address<addr::Data> {
        self.data[idx as usize]
    }

    pub fn resolve(&self, name: &str) -> Option<&ExportInstance> {
        self.exports.iter().find(|e| e.name == name)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ModuleInstanceBuilder {
    pub types:   Vec<FunctionType>,
    pub exports: Vec<ExportInstance>,
    pub funcs:   Vec<Address<addr::Function>>,
    pub tables:  Vec<Address<addr::Table>>,
    pub mems:    Vec<Address<addr::Memory>>,
    pub globals: Vec<Address<addr::Global>>,
    pub elems:   Vec<Address<addr::Elem>>,
    pub data:    Vec<Address<addr::Data>>,
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
            types:   self.types.into_boxed_slice(),
            exports: self.exports.into_boxed_slice(),
            funcs:   self.funcs.into_boxed_slice(),
            tables:  self.tables.into_boxed_slice(),
            mems:    self.mems.into_boxed_slice(),
            globals: self.globals.into_boxed_slice(),
            elems:   self.elems.into_boxed_slice(),
            data:    self.data.into_boxed_slice(),
        }
    }
}
