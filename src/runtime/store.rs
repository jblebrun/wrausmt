use super::instance::export_instance::ExternalVal;
use super::instance::ExportInstance;
use super::instance::FunctionInstance;
use super::ModuleInstance;
use crate::{
    err,
    error::Result,
    module::{ExportDesc, Module},
};
use std::rc::Rc;

/// Function instances, table instances, memory instances, and global instances,
/// element instances, and data instances in the store are referenced with
/// abstract addresses. These are simply indices into the respective store
/// component. In addition, an embedder may supply an uninterpreted set of host
/// addresses.
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
/// instances of each category:
///
///   store := {
///     funcs [FunctionInstance]*,
///     tables [TableInstance]*,
///     mems [MemoryInstance]*,
///     globals [GlobalInstance]*,
///     elems [ElemInstance]*,
///     data [DataInstance]*,
///   }
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
        if self.funcs.len() < addr as usize {
            return err!("No function at addr {}", addr);
        }

        Ok(self.funcs[addr as usize].clone())
    }

    /// Load the provided module into this store.
    /// The module will be consumed by this call, and
    /// ModuleInstance representing it will be returned.
    pub fn load(&mut self, module: Module) -> Rc<ModuleInstance> {
        // Calculate the address offset for this module.
        // We won't explicity save idx->address mapping, instead
        // it can be generated by adding this offset to the module
        // indices.
        let func_offset = self.funcs.len() as u32;
        let table_offset = 0;
        let memory_offset = 0;
        let global_offset = 0;

        let mod_inst = Rc::new(ModuleInstance {
            // Take the module types from the module for the instance.
            types: module.types,

            func_offset,

            // Convert the module exports into runtime exports;
            // this means converting the index values into address
            // values.
            exports: module
                .exports
                .into_vec()
                .into_iter()
                .map(|e| ExportInstance {
                    name: e.name,
                    addr: match e.desc {
                        ExportDesc::Func(idx) => ExternalVal::Func(idx + func_offset),
                        ExportDesc::Table(idx) => ExternalVal::Table(idx + table_offset),
                        ExportDesc::Memory(idx) => ExternalVal::Memory(idx + memory_offset),
                        ExportDesc::Global(idx) => ExternalVal::Global(idx + global_offset),
                    },
                })
                .collect(),
        });

        // Append created functions into the store.
        for f in module.funcs.into_vec() {
            self.funcs.push(Rc::new(FunctionInstance {
                module_instance: mod_inst.clone(),
                code: f,
            }));
        }

        mod_inst
    }
}
