use crate::runtime::store::addr;
use crate::module;

use super::ModuleInstance;

/// An external value is the runtime representation of an entity that can be
/// imported or exported. [Spec][Spec]
///
/// It is an address denoting either a function instance, table instance, memory
/// instance, or global instances in the shared store.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#external-values
#[derive(Debug, Clone, Copy)]
pub enum ExternalVal {
    Func(addr::FuncAddr),
    Table(addr::TableAddr),
    Memory(addr::MemoryAddr),
    Global(addr::GlobalAddr),
}

/// An export instance is the runtime representation of an export. [Spec][Spec]
///
/// It defines the export’s name and the associated external value.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#export-instances
#[derive(Debug, Clone)]
pub struct ExportInstance {
    pub name: String,
    pub addr: ExternalVal,
}

impl ExportInstance {
    /// Create a new export instance for the provided [Export][module::Export], based on the address
    /// information in the provided [ModuleInstance]. The provided [ModuleInstance]
    /// should reflect a module that's already had functions, tables, mems, and globals
    /// allocated in the store.
    pub fn new(e: module::Export, module_instance: &ModuleInstance) -> ExportInstance {
        ExportInstance {
            name: e.name,
            addr: match e.desc {
                    module::ExportDesc::Func(idx) => ExternalVal::Func(idx + module_instance.func_offset),
                    module::ExportDesc::Table(idx) => ExternalVal::Table(idx + module_instance.table_offset),
                    module::ExportDesc::Memory(idx) => ExternalVal::Memory(idx + module_instance.mem_offset),
                    module::ExportDesc::Global(idx) => ExternalVal::Global(idx + module_instance.global_offset),
            }
        }
    }
}
