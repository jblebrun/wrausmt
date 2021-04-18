use super::*;
use crate::types::FunctionType;

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
#[derive(Debug, Default)]
pub struct ModuleInstance {
    pub types: Box<[FunctionType]>,
    pub exports: Box<[ExportInstance]>,

    pub func_offset: u32,
    pub func_count: usize,

    pub table_offset: u32,
    pub table_count: usize,

    pub mem_offset: u32,
    pub global_offset: u32,
}

impl ModuleInstance {
    pub fn resolve(&self, name: &str) -> Option<&ExportInstance> {
        let found = self.exports.iter().find(|e| e.name == name);

        found
    }
}
