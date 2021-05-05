use std::collections::HashMap;

use super::resolve::{IdentifierContext, Resolve, Result};
use crate::syntax::{
    DataField, ElemField, ExportDesc, ExportField, Expr, FuncField, FunctionType, GlobalField,
    ImportDesc, ImportField, Index, MemoryField, Module, Operands, Resolved, StartField,
    TableField, TypeField, TypeUse, Unresolved,
};

#[derive(Clone, Default, Debug, PartialEq)]
pub struct ModuleIdentifiers {
    pub typeindices: HashMap<String, u32>,
    pub funcindices: HashMap<String, u32>,
    pub tableindices: HashMap<String, u32>,
    pub memindices: HashMap<String, u32>,
    pub globalindices: HashMap<String, u32>,
    pub elemindices: HashMap<String, u32>,
    pub dataindices: HashMap<String, u32>,
}

/// A [ModuleBuilder] accepts the various [Field] items coming from the parse, and organizes them
/// by sections into a [Module]. This [Module] is still just an abstract representation. ID
/// declarations  are collected into maps, but ID usages are not yet resolved. ID resolution and
/// function body compilation happens in a subsequent resolution pass.
#[derive(Debug, Default)]
pub struct ModuleBuilder {
    module: Module<Unresolved>,
    module_identifiers: ModuleIdentifiers,
    funcidx_offset: u32,
    tableidx_offset: u32,
    memidx_offset: u32,
    globalidx_offset: u32,
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
    pub fn new(id: Option<String>) -> Self {
        Self {
            module: Module {
                id,
                ..Module::default()
            },
            ..ModuleBuilder::default()
        }
    }

    pub fn build(self) -> Result<Module<Resolved>> {
        let ic = IdentifierContext::new(self.module_identifiers);
        self.module.resolve(&ic)
    }

    pub fn tables(&self) -> u32 {
        self.module.tables.len() as u32
    }

    pub fn add_typefield(&mut self, typefield: TypeField) {
        add_ident!(self, typefield, typeindices, types, 0);
        self.module.types.push(typefield);
    }

    pub fn add_inline_typeuse(&mut self, functiontype: FunctionType) -> u32 {
        let existing = self
            .module
            .types
            .iter()
            .position(|t| t.functiontype == functiontype);
        match existing {
            None => {
                let idx = self.module.types.len();
                self.module.types.push(TypeField {
                    id: None,
                    functiontype,
                });
                idx as u32
            }
            Some(idx) => idx as u32,
        }
    }

    // Resolves the type use, and updates the index of the provided typeuse if needed.
    fn resolve_typeuse(&mut self, tu: &mut TypeUse<Unresolved>) {
        if let Some(inline_typefield) = tu.get_inline_def() {
            let idx = self.add_inline_typeuse(inline_typefield);
            let mut tu = tu;
            tu.typeidx = Some(Index::unnamed(idx));
        }
    }

    // Travel through the function body looking for operations that have a typeuse.
    // When one is found, handle anonymous/inline usages if needed.
    fn resolve_func_body_typeuse(&mut self, expr: &mut Expr<Unresolved>) {
        for instr in expr.instr.iter_mut() {
            match &mut instr.operands {
                Operands::CallIndirect(_, tu) => {
                    self.resolve_typeuse(tu);
                }
                Operands::Block(_, _, e, _) => {
                    self.resolve_func_body_typeuse(e);
                }
                Operands::If(_, _, th, el) => {
                    self.resolve_func_body_typeuse(th);
                    self.resolve_func_body_typeuse(el);
                }
                _ => (),
            };
        }
    }

    pub fn add_funcfield(&mut self, f: FuncField<Unresolved>) {
        let mut f = f;
        // type use may define new type
        if let Some(inline_typefield) = f.typeuse.get_inline_def() {
            let idx = self.add_inline_typeuse(inline_typefield);
            f.typeuse.typeidx = Some(Index::unnamed(idx))
        }

        // Resolve call_indirect type usages
        self.resolve_func_body_typeuse(&mut f.body);

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

    pub fn add_tablefield(&mut self, f: TableField) {
        add_ident!(self, f, tableindices, tables, self.tableidx_offset);

        // export field may define new exports.
        let tableidx = self.module.funcs.len() as u32;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name: export_name.clone(),
                exportdesc: ExportDesc::Table(Index::unnamed(tableidx)),
            })
        }
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
        let mut f = f;
        if let ImportDesc::Func(ref mut tu) = &mut f.desc {
            self.resolve_typeuse(tu);
        }

        // Imports contribute to index counts in their corresponding
        // space, and must appear before any declarations of that type
        // in the module, so we track their counts of each type in order
        // to adjust indices.
        match f.desc {
            ImportDesc::Func(_) => self.funcidx_offset += 1,
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
