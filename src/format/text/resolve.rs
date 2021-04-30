//! Methods implementing index usage resolution.
use std::collections::HashMap;

use crate::error;
use crate::error::Result;
use super::syntax::{DataField, DataIndex, DataInit, ElemField, ElemIndex, ElemList, ExportDesc, ExportField, Expr, FuncField, FuncIndex, GlobalField, GlobalIndex, ImportDesc, ImportField, Index, Instruction, LabelIndex, LocalIndex, MemoryIndex, ModeEntry, Module, ModuleIdentifiers, Operands, Resolved, StartField, TableElems, TableField, TableIndex, TablePosition, TableUse, TypeIndex, TypeUse, Unresolved};

/// A structure to hold the currently resolvable set of identifiers.
#[derive(Debug)]
pub struct IdentifierContext<'a> {
    pub modulescope: &'a ModuleIdentifiers,
    pub localindices: &'a HashMap<String, u32>,
    pub labelindices: &'a HashMap<String, u32> 
}

/// Each syntax element that contains an index usag, an element containin an index
/// usage, should implement this trait with logic describing how to return the 
/// element in a resolved state.
pub trait Resolve<T> {
    fn resolve(self, ic: &IdentifierContext) -> Result<T>;
}

/// For an iterable of unresolved items, returns a Vector with all of the items resolved.
macro_rules! resolve_all {
    ( $dst:ident, $src:expr, $ic:expr ) => {
        let $dst: Result<Vec<_>> = $src.into_iter().map(|i| i.resolve(&$ic)).collect();

    }
}

/// For an option of an unresolved items, returns an option of the resolved item.
macro_rules! resolve_option {
    ( $dst:ident, $src:expr, $ic:expr ) => {
        let $dst = $src.map(|i| i.resolve(&$ic)).transpose()?;
    }
}

/// This generates each of the [Resolve] impls for the [Index] in each [IndexSpace].
macro_rules! index_resolver {
    ( $it:ty, $ic:ident, $src:expr  ) => {
        impl Resolve<Index<Resolved, $it>> for Index<Unresolved, $it> {
            fn resolve(self, $ic: &IdentifierContext) -> Result<Index<Resolved, $it>> {
                let value = if self.name.is_empty() {
                    self.value
                } else {
                    // TODO - how to handle the different index types?
                    let value = $src.get(&self.name)
                        .ok_or_else(|| error!("id not found {}", self.name))?;
                    *value
                };
                Ok(self.resolved(value))
            }
        }
    }
}

index_resolver!{TypeIndex, ic, ic.modulescope.typeindices}
index_resolver!{FuncIndex, ic, ic.modulescope.funcindices}
index_resolver!{TableIndex, ic, ic.modulescope.tableindices}
index_resolver!{GlobalIndex, ic, ic.modulescope.globalindices}
index_resolver!{MemoryIndex, ic, ic.modulescope.memindices}
index_resolver!{ElemIndex, ic, ic.modulescope.elemindices}
index_resolver!{DataIndex, ic, ic.modulescope.dataindices}
index_resolver!{LocalIndex, ic, ic.localindices}
index_resolver!{LabelIndex, ic, ic.labelindices}

impl Resolve<Expr<Resolved>> for Expr<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<Expr<Resolved>> {
        resolve_all!(instr, self.instr, ic);
        Ok(Expr {
            instr: instr?
        })
    }
}

impl Resolve<Instruction<Resolved>> for Instruction<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<Instruction<Resolved>> {
        Ok(Instruction{
            name: self.name,
            opcode: self.opcode,
            operands: self.operands.resolve(&ic)?
        })
    }
}

impl Resolve<Operands<Resolved>> for Operands<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<Operands<Resolved>> {
        Ok(
        match self {
            Operands::None => Operands::None,
            Operands::FuncIndex(idx) => Operands::FuncIndex(idx.resolve(&ic)?),
            Operands::TableIndex(idx) => Operands::TableIndex(idx.resolve(&ic)?),
            Operands::GlobalIndex(idx) => Operands::GlobalIndex(idx.resolve(&ic)?),
            Operands::ElemIndex(idx) => Operands::ElemIndex(idx.resolve(&ic)?),
            Operands::DataIndex(idx) => Operands::DataIndex(idx.resolve(&ic)?),
            Operands::LocalIndex(idx) => Operands::LocalIndex(idx.resolve(&ic)?),
            Operands::LabelIndex(idx) => Operands::LabelIndex(idx.resolve(&ic)?),
            Operands::Memargs(a, o) => Operands::Memargs(a, o),
            Operands::I32(v) => Operands::I32(v),
            Operands::I64(v) => Operands::I64(v),
            Operands::F32(v) => Operands::F32(v),
            Operands::F64(v) => Operands::F64(v),
        })
    }
}

impl Resolve<TableField<Resolved>> for TableField<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<TableField<Resolved>> {
        resolve_option!(elems, self.elems, ic);
        Ok(TableField {
            id: self.id,
            exports: self.exports,
            tabletype: self.tabletype,
            elems 
        })
    }
}

impl Resolve<TableElems<Resolved>> for TableElems<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<TableElems<Resolved>> {
        Ok(match self {
            TableElems::Elem(el) => TableElems::Elem(el.resolve(&ic)?),
            TableElems::Expr(exprs) => {
                resolve_all!(e, exprs, ic);
                TableElems::Expr(e?)
            }
        })
    }
}

impl Resolve<ElemList<Resolved>> for ElemList<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<ElemList<Resolved>> {
        resolve_all!(items, self.items, ic);
        Ok(ElemList{
            reftype: self.reftype,
            items: items?
        })
    }
}

impl Resolve<ImportField<Resolved>> for ImportField<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<ImportField<Resolved>> {
        Ok(ImportField {
            modname: self.modname,
            name: self.name,
            id: self.id,
            desc: self.desc.resolve(&ic)?
        })
    }
}

impl Resolve<ImportDesc<Resolved>> for ImportDesc<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<ImportDesc<Resolved>> {
        Ok(match self {
            ImportDesc::Func(tu) => ImportDesc::Func(tu.resolve(&ic)?),
            ImportDesc::Table(tt) => ImportDesc::Table(tt),
            ImportDesc::Mem(mt) => ImportDesc::Mem(mt),
            ImportDesc::Global(gt) => ImportDesc::Global(gt)
        })
    }
}

impl Resolve<ExportField<Resolved>> for ExportField<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<ExportField<Resolved>> {
        Ok(ExportField{
            name: self.name,
            exportdesc: self.exportdesc.resolve(&ic)?
        })
    }
}

impl Resolve<ExportDesc<Resolved>> for ExportDesc<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<ExportDesc<Resolved>> {
        Ok(match self {
            ExportDesc::Func(idx) => ExportDesc::Func(idx.resolve(&ic)?),
            ExportDesc::Table(idx) => ExportDesc::Table(idx.resolve(&ic)?),
            ExportDesc::Mem(idx) => ExportDesc::Mem(idx.resolve(&ic)?),
            ExportDesc::Global(idx) => ExportDesc::Global(idx.resolve(&ic)?),
        })
    }
}

impl Resolve<GlobalField<Resolved>> for GlobalField<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<GlobalField<Resolved>> {
        Ok(GlobalField{
            id: self.id,
            exports: self.exports,
            globaltype: self.globaltype,
            init: self.init.resolve(&ic)?
        })
    }
}

impl Resolve<StartField<Resolved>> for StartField<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<StartField<Resolved>> {
        Ok(StartField{
            idx: self.idx.resolve(&ic)?
        })
    }
}

impl Resolve<ElemField<Resolved>> for ElemField<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<ElemField<Resolved>> {
        Ok(ElemField{
            id: self.id,
            mode: self.mode.resolve(&ic)?,
            elemlist: self.elemlist.resolve(&ic)?
        })
    }
}

impl Resolve<ModeEntry<Resolved>> for ModeEntry<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<ModeEntry<Resolved>> {
        Ok(match self {
            ModeEntry::Passive => ModeEntry::Passive,
            ModeEntry::Active(tp) => ModeEntry::Active(tp.resolve(&ic)?),
            ModeEntry::Declarative => ModeEntry::Declarative
        })
    }
}

impl Resolve<TablePosition<Resolved>> for TablePosition<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<TablePosition<Resolved>> {
        Ok(TablePosition{
            tableuse: self.tableuse.resolve(&ic)?,
            offset: self.offset.resolve(&ic)?
        })
    }
}

impl Resolve<TableUse<Resolved>> for TableUse<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<TableUse<Resolved>> {
        Ok(TableUse{
            tableidx: self.tableidx.resolve(&ic)?
        })
    }
}

impl Resolve<DataField<Resolved>> for DataField<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<DataField<Resolved>> {
        resolve_option!(init, self.init, ic);
        Ok(DataField{
            id: self.id,
            data: self.data,
            init
        })
    }
}

impl Resolve<DataInit<Resolved>> for DataInit<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<DataInit<Resolved>> {
        Ok(DataInit {
            memidx: self.memidx.resolve(&ic)?,
            offset: self.offset.resolve(&ic)?
        })
    }
}

impl Resolve<Module<Resolved>> for Module<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<Module<Resolved>> {
        resolve_all!(funcs, self.funcs, ic);
        resolve_all!(tables, self.tables, ic);
        resolve_all!(imports, self.imports, ic);
        resolve_all!(exports, self.exports, ic);
        resolve_all!(globals, self.globals, ic);
        resolve_all!(elems, self.elems, ic);
        resolve_option!(start, self.start, ic);
        resolve_all!(data, self.data, ic);
        Ok(Module {
            id: self.id,
            types: self.types,
            funcs: funcs?,
            tables: tables?,
            memories: self.memories,
            imports: imports?,
            exports: exports?,
            globals: globals?,
            start,
            elems: elems?,
            data: data?,
            identifiers: self.identifiers
        })
    }
}

impl Resolve<TypeUse<Resolved>> for TypeUse<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<TypeUse<Resolved>> {
        resolve_option!(typeidx, self.typeidx, ic);
        Ok(TypeUse { typeidx, functiontype: self.functiontype })
    }
}

impl Resolve<FuncField<Resolved>> for FuncField<Unresolved> {
    fn resolve(self, ic: &IdentifierContext) -> Result<FuncField<Resolved>> {
        let typeuse = self.typeuse.resolve(&ic)?;

        let fic = IdentifierContext {
            modulescope: ic.modulescope,
            localindices: &self.localindices,
            labelindices: ic.labelindices
        };
        let body = self.body.resolve(&fic)?;

        Ok(FuncField {
            id: self.id,
            exports: self.exports,
            typeuse,
            locals: self.locals,
            body,
            localindices: self.localindices
        })
    }
}
