use crate::runtime::store::addr;

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
