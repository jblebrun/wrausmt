use super::super::module::*;
use super::super::types::*;
use std::rc::Rc;
use super::super::error::{Result, Error, ErrorFrom};
use super::error::ArgumentCountError;

/// The WASM Store as described in the specification.
#[derive(Debug)]
pub struct Store {
    // FunctionInstance contain Functions, and thus the code
    // to run. They will be used by execution threads, so 
    // are stored as Rc.
    pub funcs: Vec<Rc<FunctionInstance>>
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

pub type FuncAddr = u32;
pub type TableAddr = u32;
pub type MemoryAddr = u32;
pub type GlobalAddr = u32;

#[derive(Debug)]
pub enum ExternalVal {
    Func(FuncAddr),
    Table(TableAddr),
    Memory(MemoryAddr),
    Global(GlobalAddr),
}

#[derive(Debug)]
pub struct ModuleInstance {
    types: Box<[FunctionType]>,
    exports: Box<[Export]>,
    func_offset: u32
}

impl ModuleInstance {
    pub fn empty() -> ModuleInstance {
        ModuleInstance { 
            types: Box::new([]), 
            exports: Box::new([]), 
            func_offset: 0 
        }

    }
    pub fn resolve(&self, name: &str) -> Option<&Export> {
        let found = self.exports.iter().find(|e| {
            e.name == name
        });

        found
    }
}

#[derive(Debug)]
pub struct Export {
    pub name: String,
    pub addr: ExternalVal 
}

impl Store {
    pub fn new() -> Store {
        Store {
            funcs: vec![]
        }
    }

    pub fn func(&self, addr: FuncAddr) -> Result<Rc<FunctionInstance>> {
        if self.funcs.len() < addr as usize {
            return Err(Error::new(format!("No function at addr {}", addr)))
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
            exports: module.exports
                .into_vec()
                .into_iter()
                .map(|e| {
                    Export {
                        name: e.name,
                        addr: match e.desc {
                            ExportDesc::Func(idx) => ExternalVal::Func(idx + func_offset),
                            ExportDesc::Table(idx) => ExternalVal::Table(idx + table_offset),
                            ExportDesc::Memory(idx) => ExternalVal::Memory(idx + memory_offset), 
                            ExportDesc::Global(idx) => ExternalVal::Global(idx + global_offset),
                        }
                    }
                }).collect()
        });

        // Append created functions into the store.
        for f in module.funcs.into_vec() {
            self.funcs.push(Rc::new(FunctionInstance {
                module_instance: mod_inst.clone(),
                code: f
            }));
        }

        mod_inst
    }
}

/// A function entry in the store.
#[derive(Debug)]
pub struct FunctionInstance {
    /// The module instance that generated this function instance.
    pub module_instance: Rc<ModuleInstance>,

    /// The list of instructions in the function.
    pub code: Function,
}

impl FunctionInstance {
    pub fn functype(&self) -> &FunctionType {
        &self.module_instance.types[self.code.functype as usize]
    }

    pub fn validate_args(&self, args: &[Value]) -> Result<()> {
        let params_arity = self.functype().params.len();
        if params_arity != args.len() {
             return Err(ArgumentCountError::new(
                    params_arity, 
                    args.len()
            ).wrap(""))
        }
        Ok(())
        
    }
}
