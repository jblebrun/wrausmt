//! Methods implementing index usage resolution.

use {
    super::module_builder::ModuleIdentifiers,
    std::collections::HashSet,
    wrausmt_common::true_or::TrueOr,
    wrausmt_runtime::syntax::{
        DataField, DataIndex, DataInit, ElemField, ElemIndex, ElemList, ExportDesc, ExportField,
        FParam, FuncField, FuncIndex, GlobalField, GlobalIndex, Id, ImportDesc, ImportField, Index,
        Instruction, LabelIndex, LocalIndex, MemoryIndex, ModeEntry, Module, Operands, Resolved,
        StartField, TableIndex, TablePosition, TableUse, TypeField, TypeIndex, TypeUse,
        UncompiledExpr, Unresolved,
    },
};

#[derive(Debug)]
pub enum ResolveError {
    UnresolvedId(Id),
    UnresolvedLabel(Id),
    UnresolvedType(Index<Resolved, TypeIndex>),
    DuplicateTypeIndex(Id),
    DuplicateType(Id),
    DuplicateFunc(Id),
    DuplicateGlobal(Id),
    DuplicateMem(Id),
    DuplicateData(Id),
    DuplicateElem(Id),
    DuplicateTable(Id),
    DuplicateLocal(Id),
    ImportAfterFunction,
    ImportAfterGlobal,
    ImportAfterTable,
    ImportAfterMemory,
    MultipleStartSections,
}

pub type Result<T> = std::result::Result<T, ResolveError>;
/// A structure to hold the currently resolvable set of identifiers.
#[derive(Debug)]
pub struct ResolutionContext<'a> {
    pub types:        &'a mut Vec<TypeField>,
    pub modulescope:  &'a ModuleIdentifiers,
    pub localindices: Vec<Id>,
    pub labelindices: Vec<Id>,
}

impl<'a> ResolutionContext<'a> {
    pub fn typeindex(&self, name: &Id) -> Option<u32> {
        self.modulescope.typeindices.get(name).copied()
    }

    pub fn funcindex(&self, name: &Id) -> Option<u32> {
        self.modulescope.funcindices.get(name).copied()
    }

    pub fn tableindex(&self, name: &Id) -> Option<u32> {
        self.modulescope.tableindices.get(name).copied()
    }

    pub fn memindex(&self, name: &Id) -> Option<u32> {
        self.modulescope.memindices.get(name).copied()
    }

    pub fn globalindex(&self, name: &Id) -> Option<u32> {
        self.modulescope.globalindices.get(name).copied()
    }

    pub fn dataindex(&self, name: &Id) -> Option<u32> {
        self.modulescope.dataindices.get(name).copied()
    }

    pub fn elemindex(&self, name: &Id) -> Option<u32> {
        self.modulescope.elemindices.get(name).copied()
    }

    pub fn localindex(&self, name: &Id) -> Option<u32> {
        self.localindices
            .iter()
            .position(|i| i == name)
            .map(|s| s as u32)
    }

    pub fn labelindex(&self, name: &Id) -> Option<u32> {
        self.labelindices
            .iter()
            .rev()
            .position(|i| i == name)
            .map(|s| s as u32)
    }

    pub fn verify_typeindex_exists(&self, tyindex: &Index<Resolved, TypeIndex>) -> Result<()> {
        if (tyindex.value() as usize) < self.types.len() {
            Ok(())
        } else {
            Err(ResolveError::UnresolvedType(tyindex.clone()))
        }
    }

    pub fn new(modulescope: &'a ModuleIdentifiers, types: &'a mut Vec<TypeField>) -> Self {
        ResolutionContext {
            types,
            modulescope,
            localindices: vec![],
            labelindices: vec![],
        }
    }

    pub fn for_func(&mut self, li: Vec<Id>) -> ResolutionContext {
        ResolutionContext {
            types:        self.types,
            modulescope:  self.modulescope,
            localindices: li,
            labelindices: self.labelindices.clone(),
        }
    }

    pub fn with_label(&mut self, id: Id) -> ResolutionContext {
        let mut li = self.labelindices.clone();
        li.push(id);
        ResolutionContext {
            types:        self.types,
            modulescope:  self.modulescope,
            localindices: self.localindices.clone(),
            labelindices: li,
        }
    }
}

trait OrEmpty<T> {
    fn or_empty(&self) -> T;
}

impl OrEmpty<Id> for Option<Id> {
    fn or_empty(&self) -> Id {
        self.clone().unwrap_or_default()
    }
}

/// Each syntax element that contains an index usag, an element containin an
/// index usage, should implement this trait with logic describing how to return
/// the element in a resolved state.
pub trait Resolve<T> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<T>;
}

/// For an iterable of unresolved items, returns a Vector with all of the items
/// resolved.
macro_rules! resolve_all {
    ( $src:expr, $ic:expr) => {
        $src.into_iter()
            .map(|i| i.resolve($ic))
            .collect::<Result<_>>()
    };
}

/// For an option of an unresolved items, returns an option of the resolved
/// item.
macro_rules! resolve_option {
    ( $src:expr, $ic:expr) => {
        $src.map(|i| i.resolve($ic)).transpose()?
    };
}

/// This generates each of the [Resolve] impls for the [Index] in each
/// [IndexSpace].
macro_rules! index_resolver {
    ( $it:ty, $ic:ident, $src:ident [$err:ident] ) => {
        impl Resolve<Index<Resolved, $it>> for Index<Unresolved, $it> {
            fn resolve(self, $ic: &mut ResolutionContext) -> Result<Index<Resolved, $it>> {
                let value = if self.name().as_str().is_empty() {
                    self.value()
                } else {
                    // TODO - how to handle the different index types?
                    let value = $ic
                        .$src(self.name())
                        .ok_or_else(|| ResolveError::$err(self.name().to_owned()))?;
                    value
                };
                Ok(self.resolved(value))
            }
        }
    };
    ( $it:ty, $ic:ident, $src:ident ) => {
        index_resolver! { $it, $ic, $src [UnresolvedId] }
    };
}

index_resolver! {TypeIndex, ic, typeindex}
index_resolver! {FuncIndex, ic, funcindex}
index_resolver! {TableIndex, ic, tableindex}
index_resolver! {GlobalIndex, ic, globalindex}
index_resolver! {MemoryIndex, ic, memindex}
index_resolver! {ElemIndex, ic, elemindex}
index_resolver! {DataIndex, ic, dataindex}
index_resolver! {LocalIndex, ic, localindex}
index_resolver! {LabelIndex, ic, labelindex [UnresolvedLabel] }

impl Resolve<UncompiledExpr<Resolved>> for UncompiledExpr<Unresolved> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<UncompiledExpr<Resolved>> {
        let instr = resolve_all!(self.instr, ic)?;
        Ok(UncompiledExpr { instr })
    }
}

impl Resolve<Instruction<Resolved>> for Instruction<Unresolved> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<Instruction<Resolved>> {
        Ok(Instruction {
            name:     self.name,
            opcode:   self.opcode,
            operands: self.operands.resolve(ic)?,
        })
    }
}

impl Resolve<Operands<Resolved>> for Operands<Unresolved> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<Operands<Resolved>> {
        Ok(match self {
            Operands::None => Operands::None,
            Operands::If(id, tu, th, el) => {
                let mut bic = ic.with_label(id.or_empty());
                let tu = tu.resolve(&mut bic)?;
                let th = th.resolve(&mut bic)?;
                let el = el.resolve(&mut bic)?;
                Operands::If(id, tu, th, el)
            }
            Operands::BrTable(idxs, last) => {
                Operands::BrTable(resolve_all!(idxs, ic)?, last.resolve(ic)?)
            }
            Operands::Select(r) => Operands::Select(r),
            Operands::CallIndirect(idx, tu) => {
                let idx = idx.resolve(ic)?;
                let tu = tu.resolve(ic)?;
                Operands::CallIndirect(idx, tu)
            }
            Operands::Block(id, tu, expr, cnt) => {
                let mut bic = ic.with_label(id.or_empty());
                let tu = tu.resolve(&mut bic)?;
                let expr = expr.resolve(&mut bic)?;
                Operands::Block(id, tu, expr, cnt)
            }
            Operands::FuncIndex(idx) => Operands::FuncIndex(idx.resolve(ic)?),
            Operands::TableIndex(idx) => Operands::TableIndex(idx.resolve(ic)?),
            Operands::GlobalIndex(idx) => Operands::GlobalIndex(idx.resolve(ic)?),
            Operands::ElemIndex(idx) => Operands::ElemIndex(idx.resolve(ic)?),
            Operands::DataIndex(idx) => Operands::DataIndex(idx.resolve(ic)?),
            Operands::LocalIndex(idx) => Operands::LocalIndex(idx.resolve(ic)?),
            Operands::LabelIndex(idx) => Operands::LabelIndex(idx.resolve(ic)?),
            Operands::MemoryIndex(idx) => Operands::MemoryIndex(idx.resolve(ic)?),
            Operands::TableInit(tidx, eidx) => {
                Operands::TableInit(tidx.resolve(ic)?, eidx.resolve(ic)?)
            }
            Operands::TableCopy(tidx, t2idx) => {
                Operands::TableCopy(tidx.resolve(ic)?, t2idx.resolve(ic)?)
            }
            Operands::Memargs(a, o) => Operands::Memargs(a, o),
            Operands::HeapType(r) => Operands::HeapType(r),
            Operands::I32(v) => Operands::I32(v),
            Operands::I64(v) => Operands::I64(v),
            Operands::F32(v) => Operands::F32(v),
            Operands::F64(v) => Operands::F64(v),
        })
    }
}

impl Resolve<ElemList<UncompiledExpr<Resolved>>> for ElemList<UncompiledExpr<Unresolved>> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<ElemList<UncompiledExpr<Resolved>>> {
        let items = resolve_all!(self.items, ic)?;
        Ok(ElemList {
            reftype: self.reftype,
            items,
        })
    }
}

impl Resolve<ImportField<Resolved>> for ImportField<Unresolved> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<ImportField<Resolved>> {
        Ok(ImportField {
            modname: self.modname,
            name:    self.name,
            id:      self.id,
            exports: self.exports,
            desc:    self.desc.resolve(ic)?,
        })
    }
}

impl Resolve<ImportDesc<Resolved>> for ImportDesc<Unresolved> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<ImportDesc<Resolved>> {
        Ok(match self {
            ImportDesc::Func(tu) => ImportDesc::Func(tu.resolve(ic)?),
            ImportDesc::Table(tt) => ImportDesc::Table(tt),
            ImportDesc::Mem(mt) => ImportDesc::Mem(mt),
            ImportDesc::Global(gt) => ImportDesc::Global(gt),
        })
    }
}

impl Resolve<ExportField<Resolved>> for ExportField<Unresolved> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<ExportField<Resolved>> {
        Ok(ExportField {
            name:       self.name,
            exportdesc: self.exportdesc.resolve(ic)?,
        })
    }
}

impl Resolve<ExportDesc<Resolved>> for ExportDesc<Unresolved> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<ExportDesc<Resolved>> {
        Ok(match self {
            ExportDesc::Func(idx) => ExportDesc::Func(idx.resolve(ic)?),
            ExportDesc::Table(idx) => ExportDesc::Table(idx.resolve(ic)?),
            ExportDesc::Mem(idx) => ExportDesc::Mem(idx.resolve(ic)?),
            ExportDesc::Global(idx) => ExportDesc::Global(idx.resolve(ic)?),
        })
    }
}

impl Resolve<GlobalField<UncompiledExpr<Resolved>>> for GlobalField<UncompiledExpr<Unresolved>> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<GlobalField<UncompiledExpr<Resolved>>> {
        Ok(GlobalField {
            id:         self.id,
            exports:    self.exports,
            globaltype: self.globaltype,
            init:       self.init.resolve(ic)?,
        })
    }
}

impl Resolve<StartField<Resolved>> for StartField<Unresolved> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<StartField<Resolved>> {
        Ok(StartField {
            idx: self.idx.resolve(ic)?,
        })
    }
}

impl Resolve<ElemField<Resolved, UncompiledExpr<Resolved>>>
    for ElemField<Unresolved, UncompiledExpr<Unresolved>>
{
    fn resolve(
        self,
        ic: &mut ResolutionContext,
    ) -> Result<ElemField<Resolved, UncompiledExpr<Resolved>>> {
        Ok(ElemField {
            id:       self.id,
            mode:     self.mode.resolve(ic)?,
            elemlist: self.elemlist.resolve(ic)?,
        })
    }
}

impl Resolve<ModeEntry<Resolved, UncompiledExpr<Resolved>>>
    for ModeEntry<Unresolved, UncompiledExpr<Unresolved>>
{
    fn resolve(
        self,
        ic: &mut ResolutionContext,
    ) -> Result<ModeEntry<Resolved, UncompiledExpr<Resolved>>> {
        Ok(match self {
            ModeEntry::Passive => ModeEntry::Passive,
            ModeEntry::Active(tp) => ModeEntry::Active(tp.resolve(ic)?),
            ModeEntry::Declarative => ModeEntry::Declarative,
        })
    }
}

impl Resolve<TablePosition<Resolved, UncompiledExpr<Resolved>>>
    for TablePosition<Unresolved, UncompiledExpr<Unresolved>>
{
    fn resolve(
        self,
        ic: &mut ResolutionContext,
    ) -> Result<TablePosition<Resolved, UncompiledExpr<Resolved>>> {
        Ok(TablePosition {
            tableuse: self.tableuse.resolve(ic)?,
            offset:   self.offset.resolve(ic)?,
        })
    }
}

impl Resolve<TableUse<Resolved>> for TableUse<Unresolved> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<TableUse<Resolved>> {
        Ok(TableUse {
            tableidx: self.tableidx.resolve(ic)?,
        })
    }
}

impl Resolve<DataField<Resolved, UncompiledExpr<Resolved>>>
    for DataField<Unresolved, UncompiledExpr<Unresolved>>
{
    fn resolve(
        self,
        ic: &mut ResolutionContext,
    ) -> Result<DataField<Resolved, UncompiledExpr<Resolved>>> {
        let init = resolve_option!(self.init, ic);
        Ok(DataField {
            id: self.id,
            data: self.data,
            init,
        })
    }
}

impl Resolve<DataInit<Resolved, UncompiledExpr<Resolved>>>
    for DataInit<Unresolved, UncompiledExpr<Unresolved>>
{
    fn resolve(
        self,
        ic: &mut ResolutionContext,
    ) -> Result<DataInit<Resolved, UncompiledExpr<Resolved>>> {
        Ok(DataInit {
            memidx: self.memidx.resolve(ic)?,
            offset: self.offset.resolve(ic)?,
        })
    }
}

pub trait ResolveModule {
    fn resolve(
        self,
        idents: &ModuleIdentifiers,
    ) -> Result<Module<Resolved, UncompiledExpr<Resolved>>>;
}

impl ResolveModule for Module<Unresolved, UncompiledExpr<Unresolved>> {
    fn resolve(
        mut self,
        mi: &ModuleIdentifiers,
    ) -> Result<Module<Resolved, UncompiledExpr<Resolved>>> {
        let mut rc = ResolutionContext::new(mi, &mut self.types);
        let customs = self.customs;
        let funcs = resolve_all!(self.funcs, &mut rc)?;
        let imports = resolve_all!(self.imports, &mut rc)?;
        let exports = resolve_all!(self.exports, &mut rc)?;
        let globals = resolve_all!(self.globals, &mut rc)?;
        let elems = resolve_all!(self.elems, &mut rc)?;
        let start = resolve_option!(self.start, &mut rc);
        let data = resolve_all!(self.data, &mut rc)?;

        Ok(Module {
            id: self.id,
            customs,
            types: self.types,
            funcs,
            tables: self.tables,
            memories: self.memories,
            imports,
            exports,
            globals,
            start,
            elems,
            data,
        })
    }
}

impl Resolve<TypeUse<Resolved>> for TypeUse<Unresolved> {
    fn resolve(self, ic: &mut ResolutionContext) -> Result<TypeUse<Resolved>> {
        validate_inline_typeuse(&self, ic)?;
        match self {
            TypeUse::ByIndex(idx) => {
                let idx = idx.resolve(ic)?;
                // We don't verify type index exists here because it causes parse error when we
                // want invalid error. func.wast line 435.
                // TODO - figure out if this can be clearer.
                Ok(TypeUse::ByIndex(idx))
            }
            TypeUse::NamedInline {
                index,
                functiontype,
            } => {
                let index = index.resolve(ic)?;
                ic.verify_typeindex_exists(&index)?;
                Ok(TypeUse::NamedInline {
                    functiontype,
                    index,
                })
            }
            TypeUse::AnonymousInline(functiontype) => {
                // Creating a new inline use if a matching type doesn't exist.
                let existing = ic
                    .types
                    .iter()
                    .position(|t| t.functiontype.anonymously_equals(&functiontype));

                let index: Index<Resolved, TypeIndex> = match existing {
                    Some(existing) => Index::unnamed(existing as u32),
                    None => {
                        let newidx = ic.types.len();
                        ic.types.push(TypeField {
                            id:           None,
                            functiontype: functiontype.clone(),
                        });
                        Index::unnamed(newidx as u32)
                    }
                };
                // We no longer need to carry the function data along.
                Ok(TypeUse::NamedInline {
                    functiontype,
                    index,
                })
            }
        }
    }
}

fn get_func_params(typeuse: &TypeUse<Resolved>, types: &[TypeField]) -> Vec<FParam> {
    match typeuse {
        TypeUse::AnonymousInline(functiontype) | TypeUse::NamedInline { functiontype, .. } => {
            functiontype.params.clone()
        }
        _ => {
            let existing = types
                .get(typeuse.index().value() as usize)
                .map(|tf| tf.functiontype.clone())
                .unwrap_or_default();
            existing.params
        }
    }
}

// Verifies that the incoming `typeuse` doesn't say anything that contradicts an
// already-existing type.
fn validate_inline_typeuse(typeuse: &TypeUse<Unresolved>, ic: &ResolutionContext) -> Result<()> {
    // We only need to check something if the incoming inline typeuse defined a
    // function and an index explicitly.
    let (new_typeidx, new_functiontype) = match (typeuse.index(), typeuse.function_type()) {
        (Some(ti), Some(ft)) if !ti.name().as_bytes().is_empty() => (ti, ft),
        _ => return Ok(()),
    };

    // If a type doesn't exist, then one is (hopefully) being created. That
    // gets checked elsewhere.
    let existing_functiontype = match ic.typeindex(new_typeidx.name()) {
        Some(ei) => &ic.types[ei as usize].functiontype,
        _ => return Ok(()),
    };

    // If no params/results were in the inline def, the existing type doesn't
    // need to be void, since in that case, the (type $i) is just reference
    // type i as anything.
    if !new_functiontype.matches_existing(existing_functiontype) {
        Err(ResolveError::DuplicateTypeIndex(new_typeidx.name().clone()))
    } else {
        Ok(())
    }
}

impl Resolve<FuncField<Resolved, UncompiledExpr<Resolved>>>
    for FuncField<Unresolved, UncompiledExpr<Unresolved>>
{
    fn resolve(
        self,
        ic: &mut ResolutionContext,
    ) -> Result<FuncField<Resolved, UncompiledExpr<Resolved>>> {
        validate_inline_typeuse(&self.typeuse, ic)?;

        let typeuse = self.typeuse.resolve(ic)?;
        let params = get_func_params(&typeuse, ic.types);

        let localindices: Vec<_> = params
            .iter()
            .map(|fp| fp.id.or_empty())
            .chain(self.locals.iter().map(|l| l.id.or_empty()))
            .collect();

        let mut idset: HashSet<&Id> = HashSet::new();
        for id in localindices.iter().filter(|id| !id.as_str().is_empty()) {
            idset
                .insert(id)
                .true_or_else(|| ResolveError::DuplicateLocal(id.clone()))?;
        }

        let body = {
            let mut fic = ic.for_func(localindices);
            self.body.resolve(&mut fic)?
        };

        Ok(FuncField {
            id: self.id,
            exports: self.exports,
            typeuse: TypeUse::ByIndex(typeuse.index().clone()),
            locals: self.locals,
            body,
        })
    }
}
