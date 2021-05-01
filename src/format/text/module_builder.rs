use std::collections::HashMap;

use super::{resolve::{IdentifierContext, Resolve}, syntax::{
    DataField, ElemField, ExportDesc, ExportField, FuncField, FunctionType, GlobalField,
    ImportDesc, ImportField, Index, MemoryField, Module, Resolved, StartField, TableField,
    TypeField, Unresolved
}};
use crate::error::Result;

/// A [ModuleBuilder] accepts the various [Field] items coming from the parse, and organizes them
/// by sections into a [Module]. This [Module] is still just an abstract representation. ID
/// declarations  are collected into maps, but ID usages are not yet resolved. ID resolution and
/// function body compilation happens in a subsequent resolution pass.
#[derive(Debug, Default)]
pub struct ModuleBuilder {
    module: Module<Unresolved>,
    funcidx_offset: u32,
    tableidx_offset: u32,
    memidx_offset: u32,
    globalidx_offset: u32,
}

macro_rules! add_ident {
    ( $self:ident, $field:ident, $dst:ident, $src:ident, $offset:expr) => {
        if let Some(id) = &$field.id {
            $self
                .module
                .identifiers
                .$dst
                .insert(id.clone(), $self.module.$src.len() as u32 + $offset);
        }
    };
}

impl ModuleBuilder {
    pub fn new(id: Option<String>) -> Self {
        Self {
            module: Module {
                id,
                ..Module::default()
            },
            ..ModuleBuilder::default()
        }
    }

    pub fn build(mut self) -> Result<Module<Resolved>> {
        // This is just a placeholder until resolution is implemented.
        // Since the ResolvedState is only used as a PhantomData marker,
        // the bit pattern is the same.
        // This is indeed unsafe; the symbolic indexes will not have been resolved, so any
        // references to them in the module body will be incorrect.
        let empty: HashMap<String, u32> = HashMap::default();
        let modulescope = std::mem::take(&mut self.module.identifiers);
        let ic = IdentifierContext {
            modulescope: &modulescope,
            localindices: &empty,
            labelindices: &empty
        };
        self.module.resolve(&ic)
    }

    pub fn add_typefield(&mut self, typefield: TypeField) {
        add_ident!(self, typefield, typeindices, types, 0);
        self.module.types.push(typefield);
    }

    pub fn add_inline_typeuse(&mut self, functiontype: FunctionType) {
        if self
            .module
            .types
            .iter()
            .position(|t| t.functiontype == functiontype)
            .is_none()
        {
            self.module.types.push(TypeField {
                id: None,
                functiontype,
            })
        }
    }

    pub fn add_funcfield(&mut self, f: FuncField<Unresolved>) {
        // type use may define new type
        if let Some(inline_typefield) = f.typeuse.get_inline_def() {
            self.add_inline_typeuse(inline_typefield)
        }

        add_ident!(self, f, funcindices, funcs, self.funcidx_offset);

        // export field may define new exports.
        let funcidx = self.module.funcs.len() as u32;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name: export_name.clone(),
                exportdesc: ExportDesc::Func(Index::unnamed(funcidx)),
            })
        }
        self.module.funcs.push(f);
    }

    pub fn add_tablefield(&mut self, f: TableField<Unresolved>) {
        add_ident!(self, f, tableindices, tables, self.tableidx_offset);

        // export field may define new exports.
        let tableidx = self.module.funcs.len() as u32;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name: export_name.clone(),
                exportdesc: ExportDesc::Table(Index::unnamed(tableidx)),
            })
        }
        // TODO elem contents may define new elem
        self.module.tables.push(f);
    }

    pub fn add_memoryfield(&mut self, f: MemoryField) {
        add_ident!(self, f, memindices, memories, self.memidx_offset);

        // export field may define new exports.
        let memidx = self.module.funcs.len() as u32;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name: export_name.clone(),
                exportdesc: ExportDesc::Mem(Index::unnamed(memidx)),
            })
        }
        self.module.memories.push(f);
        // data contents may define new data
    }

    pub fn add_importfield(&mut self, f: ImportField<Unresolved>) {
        // Imports contribute to index counts in their corresponding
        // space, and must appear before any declarations of that type
        // in the module, so we track their counts of each type in order
        // to adjust indices.
        match &f.desc {
            ImportDesc::Func(tu) => {
                // Function import may define a new type.
                if let Some(inline_typefield) = tu.get_inline_def() {
                    self.add_inline_typeuse(inline_typefield)
                }
                self.funcidx_offset += 1
            }
            ImportDesc::Mem(_) => self.memidx_offset += 1,
            ImportDesc::Table(_) => self.tableidx_offset += 1,
            ImportDesc::Global(_) => self.globalidx_offset += 1,
        }
        self.module.imports.push(f);
    }

    pub fn add_exportfield(&mut self, f: ExportField<Unresolved>) {
        self.module.exports.push(f)
    }

    pub fn add_globalfield(&mut self, f: GlobalField<Unresolved>) {
        add_ident!(self, f, globalindices, globals, self.globalidx_offset);

        let globalidx = self.module.funcs.len() as u32;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name: export_name.clone(),
                exportdesc: ExportDesc::Mem(Index::unnamed(globalidx)),
            })
        }
        // export field may define new exports.
        self.module.globals.push(f);
    }

    pub fn add_startfield(&mut self, f: StartField<Unresolved>) {
        self.module.start = Some(f)
    }

    pub fn add_elemfield(&mut self, f: ElemField<Unresolved>) {
        add_ident!(self, f, elemindices, elems, 0);

        self.module.elems.push(f)
    }

    pub fn add_datafield(&mut self, f: DataField<Unresolved>) {
        add_ident!(self, f, dataindices, data, 0);

        self.module.data.push(f)
    }
}
