use {
    super::resolve::{ResolveError, ResolveModule, Result},
    std::collections::HashMap,
    wrausmt_common::true_or::TrueOr,
    wrausmt_runtime::syntax::{
        DataField, ElemField, ExportDesc, ExportField, FuncField, GlobalField, Id, ImportDesc,
        ImportField, Index, MemoryField, Module, Resolved, StartField, TableField, TypeField,
        Unresolved,
    },
};

#[derive(Clone, Default, Debug, PartialEq)]
pub struct ModuleIdentifiers {
    pub typeindices:   HashMap<Id, u32>,
    pub funcindices:   HashMap<Id, u32>,
    pub tableindices:  HashMap<Id, u32>,
    pub memindices:    HashMap<Id, u32>,
    pub globalindices: HashMap<Id, u32>,
    pub elemindices:   HashMap<Id, u32>,
    pub dataindices:   HashMap<Id, u32>,
}

/// A [ModuleBuilder] accepts the various items coming from the parse, and
/// organizes them by sections into a [Module]. This [Module] is still just an
/// abstract representation. ID declarations  are collected into maps, but ID
/// usages are not yet resolved. ID resolution and function body compilation
/// happens in a subsequent resolution pass.
#[derive(Debug, Default)]
pub struct ModuleBuilder {
    module:             Module<Unresolved>,
    module_identifiers: ModuleIdentifiers,
    funcidx_offset:     u32,
    tableidx_offset:    u32,
    memidx_offset:      u32,
    globalidx_offset:   u32,
}

macro_rules! add_ident {
    ( $self:ident, $field:ident, $dst:ident, $src:ident, $offset:expr) => {
        if let Some(id) = &$field.id {
            $self
                .module_identifiers
                .$dst
                .insert(id.clone(), $self.module.$src.len() as u32 + $offset);
        }
    };
}

impl ModuleBuilder {
    pub fn new(id: Option<Id>) -> Self {
        Self {
            module: Module {
                id,
                ..Module::default()
            },
            ..ModuleBuilder::default()
        }
    }

    pub fn empty(&self) -> bool {
        self.module.types.is_empty()
            && self.module.funcs.is_empty()
            && self.module.tables.is_empty()
            && self.module.memories.is_empty()
            && self.module.imports.is_empty()
            && self.module.exports.is_empty()
            && self.module.globals.is_empty()
            && self.module.start.is_none()
            && self.module.elems.is_empty()
            && self.module.data.is_empty()
    }

    pub fn build(self) -> Result<Module<Resolved>> {
        let mod_idents = self.module_identifiers;
        self.module.resolve(mod_idents)
    }

    pub fn tables(&self) -> u32 {
        self.module.tables.len() as u32
    }

    pub fn memories(&self) -> u32 {
        self.module.memories.len() as u32
    }

    pub fn add_typefield(&mut self, typefield: TypeField) {
        add_ident!(self, typefield, typeindices, types, 0);
        self.module.types.push(typefield);
    }

    pub fn add_funcfield(&mut self, f: FuncField<Unresolved>) -> Result<()> {
        add_ident!(self, f, funcindices, funcs, self.funcidx_offset);
        // self.validate_inline_typeuse(&f.typeuse)?;

        // export field may define new exports.
        let funcidx = self.module.funcs.len() as u32 + self.funcidx_offset;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name:       export_name.clone(),
                exportdesc: ExportDesc::Func(Index::unnamed(funcidx)),
            })
        }
        self.module.funcs.push(f);
        Ok(())
    }

    pub fn add_tablefield(&mut self, f: TableField) {
        add_ident!(self, f, tableindices, tables, self.tableidx_offset);

        // export field may define new exports.
        let tableidx = self.module.tables.len() as u32 + self.tableidx_offset;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name:       export_name.clone(),
                exportdesc: ExportDesc::Table(Index::unnamed(tableidx)),
            })
        }
        self.module.tables.push(f);
    }

    pub fn add_memoryfield(&mut self, f: MemoryField) {
        add_ident!(self, f, memindices, memories, self.memidx_offset);

        // export field may define new exports.
        let memidx = self.module.memories.len() as u32 + self.memidx_offset;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name:       export_name.clone(),
                exportdesc: ExportDesc::Mem(Index::unnamed(memidx)),
            })
        }
        self.module.memories.push(f);
        // data contents may define new data
    }

    pub fn add_importfield(&mut self, f: ImportField<Unresolved>) -> Result<()> {
        (self.module.funcs.is_empty()).true_or(ResolveError::ImportAfterFunction)?;
        (self.module.globals.is_empty()).true_or(ResolveError::ImportAfterGlobal)?;
        (self.module.memories.is_empty()).true_or(ResolveError::ImportAfterMemory)?;
        (self.module.tables.is_empty()).true_or(ResolveError::ImportAfterTable)?;

        // Imports contribute to index counts in their corresponding
        // space, and must appear before any declarations of that type
        // in the module, so we track their counts of each type in order
        // to adjust indices.
        match f.desc {
            ImportDesc::Func(_) => {
                add_ident!(self, f, funcindices, funcs, self.funcidx_offset);
                for export_name in &f.exports {
                    self.module.exports.push(ExportField {
                        name:       export_name.clone(),
                        exportdesc: ExportDesc::Func(Index::unnamed(self.funcidx_offset)),
                    })
                }
                self.funcidx_offset += 1;
            }
            ImportDesc::Mem(_) => {
                add_ident!(self, f, memindices, memories, self.memidx_offset);
                for export_name in &f.exports {
                    self.module.exports.push(ExportField {
                        name:       export_name.clone(),
                        exportdesc: ExportDesc::Mem(Index::unnamed(self.memidx_offset)),
                    })
                }
                self.memidx_offset += 1;
            }
            ImportDesc::Table(_) => {
                add_ident!(self, f, tableindices, tables, self.tableidx_offset);
                for export_name in &f.exports {
                    self.module.exports.push(ExportField {
                        name:       export_name.clone(),
                        exportdesc: ExportDesc::Table(Index::unnamed(self.tableidx_offset)),
                    })
                }
                self.tableidx_offset += 1;
            }
            ImportDesc::Global(_) => {
                add_ident!(self, f, globalindices, globals, self.globalidx_offset);
                for export_name in &f.exports {
                    self.module.exports.push(ExportField {
                        name:       export_name.clone(),
                        exportdesc: ExportDesc::Global(Index::unnamed(self.globalidx_offset)),
                    })
                }
                self.globalidx_offset += 1;
            }
        }
        self.module.imports.push(f);
        Ok(())
    }

    pub fn add_exportfield(&mut self, f: ExportField<Unresolved>) {
        self.module.exports.push(f)
    }

    pub fn add_globalfield(&mut self, f: GlobalField<Unresolved>) {
        add_ident!(self, f, globalindices, globals, self.globalidx_offset);

        let globalidx = self.module.globals.len() as u32 + self.globalidx_offset;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name:       export_name.clone(),
                exportdesc: ExportDesc::Global(Index::unnamed(globalidx)),
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
