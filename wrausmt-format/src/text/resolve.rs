//! Methods implementing index usage resolution.
use {
    super::module_builder::ModuleIdentifiers,
    wrausmt_runtime::syntax::{
        DataField, DataIndex, DataInit, ElemField, ElemIndex, ElemList, ExportDesc, ExportField,
        Expr, FParam, FuncField, FuncIndex, GlobalField, GlobalIndex, Id, ImportDesc, ImportField,
        Index, Instruction, LabelIndex, LocalIndex, MemoryIndex, ModeEntry, Module, Operands,
        Resolved, StartField, TableIndex, TablePosition, TableUse, TypeField, TypeIndex, TypeUse,
        Unresolved,
    },
};

#[derive(Debug)]
pub enum ResolveError {
    UnresolvedIndex(Id),
    UnresolvedType(Index<Resolved, TypeIndex>),
    DuplicateTypeIndex(Id),
    ImportAfterFunction,
    ImportAfterGlobal,
    ImportAfterTable,
    ImportAfterMemory,
}

pub type Result<T> = std::result::Result<T, ResolveError>;
/// A structure to hold the currently resolvable set of identifiers.
#[derive(Debug)]
pub struct ResolutionContext {
    pub modulescope:  ModuleIdentifiers,
    pub localindices: Vec<Id>,
    pub labelindices: Vec<Id>,
}

impl ResolutionContext {
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

    pub fn new(modulescope: ModuleIdentifiers) -> Self {
        ResolutionContext {
            modulescope,
            localindices: Vec::new(),
            labelindices: Vec::new(),
        }
    }

    pub fn for_func(&self, li: Vec<Id>) -> Self {
        Self {
            modulescope:  self.modulescope.clone(),
            localindices: li,
            labelindices: self.labelindices.clone(),
        }
    }

    pub fn with_label(&self, id: Id) -> ResolutionContext {
        let mut li = self.labelindices.clone();
        li.push(id);
        ResolutionContext {
            modulescope:  self.modulescope.clone(),
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
    fn resolve(self, ic: &ResolutionContext, types: &mut Vec<TypeField>) -> Result<T>;
}

/// For an iterable of unresolved items, returns a Vector with all of the items
/// resolved.
macro_rules! resolve_all {
    ( $src:expr, $ic:expr, $types:expr ) => {
        $src.into_iter()
            .map(|i| i.resolve(&$ic, $types))
            .collect::<Result<_>>()
    };
}

/// For an option of an unresolved items, returns an option of the resolved
/// item.
macro_rules! resolve_option {
    ( $src:expr, $ic:expr, $types:expr ) => {
        $src.map(|i| i.resolve(&$ic, $types)).transpose()?
    };
}

/// This generates each of the [Resolve] impls for the [Index] in each
/// [IndexSpace].
macro_rules! index_resolver {
    ( $it:ty, $ic:ident, $src:ident ) => {
        impl Resolve<Index<Resolved, $it>> for Index<Unresolved, $it> {
            fn resolve(
                self,
                $ic: &ResolutionContext,
                _: &mut Vec<TypeField>,
            ) -> Result<Index<Resolved, $it>> {
                let value = if self.name().as_str().is_empty() {
                    self.value()
                } else {
                    // TODO - how to handle the different index types?
                    let value = $ic
                        .$src(self.name())
                        .ok_or_else(|| ResolveError::UnresolvedIndex(self.name().to_owned()))?;
                    value
                };
                Ok(self.resolved(value))
            }
        }
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
index_resolver! {LabelIndex, ic, labelindex}

impl Resolve<Expr<Resolved>> for Expr<Unresolved> {
    fn resolve(self, ic: &ResolutionContext, types: &mut Vec<TypeField>) -> Result<Expr<Resolved>> {
        let instr = resolve_all!(self.instr, ic, types)?;
        Ok(Expr { instr })
    }
}

impl Resolve<Instruction<Resolved>> for Instruction<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<Instruction<Resolved>> {
        Ok(Instruction {
            name:     self.name,
            opcode:   self.opcode,
            operands: self.operands.resolve(ic, types)?,
        })
    }
}

impl Resolve<Operands<Resolved>> for Operands<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<Operands<Resolved>> {
        Ok(match self {
            Operands::None => Operands::None,
            Operands::If(id, tu, th, el) => {
                let bic = ic.with_label(id.or_empty());
                let tu = tu.resolve(&bic, types)?;
                let th = th.resolve(&bic, types)?;
                let el = el.resolve(&bic, types)?;
                Operands::If(id, tu, th, el)
            }
            Operands::BrTable(idxs) => Operands::BrTable(resolve_all!(idxs, ic, types)?),
            Operands::Select(r) => Operands::Select(r),
            Operands::CallIndirect(idx, tu) => {
                let idx = idx.resolve(ic, types)?;
                let tu = tu.resolve(ic, types)?;
                Operands::CallIndirect(idx, tu)
            }
            Operands::Block(id, tu, expr, cnt) => {
                let bic = ic.with_label(id.or_empty());
                let tu = tu.resolve(&bic, types)?;
                let expr = expr.resolve(&bic, types)?;
                Operands::Block(id, tu, expr, cnt)
            }
            Operands::FuncIndex(idx) => Operands::FuncIndex(idx.resolve(ic, types)?),
            Operands::TableIndex(idx) => Operands::TableIndex(idx.resolve(ic, types)?),
            Operands::GlobalIndex(idx) => Operands::GlobalIndex(idx.resolve(ic, types)?),
            Operands::ElemIndex(idx) => Operands::ElemIndex(idx.resolve(ic, types)?),
            Operands::DataIndex(idx) => Operands::DataIndex(idx.resolve(ic, types)?),
            Operands::LocalIndex(idx) => Operands::LocalIndex(idx.resolve(ic, types)?),
            Operands::LabelIndex(idx) => Operands::LabelIndex(idx.resolve(ic, types)?),
            Operands::MemoryIndex(idx) => Operands::MemoryIndex(idx.resolve(ic, types)?),
            Operands::TableInit(tidx, eidx) => {
                Operands::TableInit(tidx.resolve(ic, types)?, eidx.resolve(ic, types)?)
            }
            Operands::TableCopy(tidx, t2idx) => {
                Operands::TableCopy(tidx.resolve(ic, types)?, t2idx.resolve(ic, types)?)
            }
            Operands::Memargs1(a, o) => Operands::Memargs1(a, o),
            Operands::Memargs2(a, o) => Operands::Memargs2(a, o),
            Operands::Memargs4(a, o) => Operands::Memargs4(a, o),
            Operands::Memargs8(a, o) => Operands::Memargs8(a, o),
            Operands::HeapType(r) => Operands::HeapType(r),
            Operands::I32(v) => Operands::I32(v),
            Operands::I64(v) => Operands::I64(v),
            Operands::F32(v) => Operands::F32(v),
            Operands::F64(v) => Operands::F64(v),
        })
    }
}

impl Resolve<ElemList<Resolved>> for ElemList<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<ElemList<Resolved>> {
        let items = resolve_all!(self.items, ic, types)?;
        Ok(ElemList {
            reftype: self.reftype,
            items,
        })
    }
}

impl Resolve<ImportField<Resolved>> for ImportField<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<ImportField<Resolved>> {
        Ok(ImportField {
            modname: self.modname,
            name:    self.name,
            id:      self.id,
            exports: self.exports,
            desc:    self.desc.resolve(ic, types)?,
        })
    }
}

impl Resolve<ImportDesc<Resolved>> for ImportDesc<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<ImportDesc<Resolved>> {
        Ok(match self {
            ImportDesc::Func(tu) => ImportDesc::Func(tu.resolve(ic, types)?),
            ImportDesc::Table(tt) => ImportDesc::Table(tt),
            ImportDesc::Mem(mt) => ImportDesc::Mem(mt),
            ImportDesc::Global(gt) => ImportDesc::Global(gt),
        })
    }
}

impl Resolve<ExportField<Resolved>> for ExportField<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<ExportField<Resolved>> {
        Ok(ExportField {
            name:       self.name,
            exportdesc: self.exportdesc.resolve(ic, types)?,
        })
    }
}

impl Resolve<ExportDesc<Resolved>> for ExportDesc<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<ExportDesc<Resolved>> {
        Ok(match self {
            ExportDesc::Func(idx) => ExportDesc::Func(idx.resolve(ic, types)?),
            ExportDesc::Table(idx) => ExportDesc::Table(idx.resolve(ic, types)?),
            ExportDesc::Mem(idx) => ExportDesc::Mem(idx.resolve(ic, types)?),
            ExportDesc::Global(idx) => ExportDesc::Global(idx.resolve(ic, types)?),
        })
    }
}

impl Resolve<GlobalField<Resolved>> for GlobalField<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<GlobalField<Resolved>> {
        Ok(GlobalField {
            id:         self.id,
            exports:    self.exports,
            globaltype: self.globaltype,
            init:       self.init.resolve(ic, types)?,
        })
    }
}

impl Resolve<StartField<Resolved>> for StartField<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<StartField<Resolved>> {
        Ok(StartField {
            idx: self.idx.resolve(ic, types)?,
        })
    }
}

impl Resolve<ElemField<Resolved>> for ElemField<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<ElemField<Resolved>> {
        Ok(ElemField {
            id:       self.id,
            mode:     self.mode.resolve(ic, types)?,
            elemlist: self.elemlist.resolve(ic, types)?,
        })
    }
}

impl Resolve<ModeEntry<Resolved>> for ModeEntry<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<ModeEntry<Resolved>> {
        Ok(match self {
            ModeEntry::Passive => ModeEntry::Passive,
            ModeEntry::Active(tp) => ModeEntry::Active(tp.resolve(ic, types)?),
            ModeEntry::Declarative => ModeEntry::Declarative,
        })
    }
}

impl Resolve<TablePosition<Resolved>> for TablePosition<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<TablePosition<Resolved>> {
        Ok(TablePosition {
            tableuse: self.tableuse.resolve(ic, types)?,
            offset:   self.offset.resolve(ic, types)?,
        })
    }
}

impl Resolve<TableUse<Resolved>> for TableUse<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<TableUse<Resolved>> {
        Ok(TableUse {
            tableidx: self.tableidx.resolve(ic, types)?,
        })
    }
}

impl Resolve<DataField<Resolved>> for DataField<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<DataField<Resolved>> {
        let init = resolve_option!(self.init, ic, types);
        Ok(DataField {
            id: self.id,
            data: self.data,
            init,
        })
    }
}

impl Resolve<DataInit<Resolved>> for DataInit<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<DataInit<Resolved>> {
        Ok(DataInit {
            memidx: self.memidx.resolve(ic, types)?,
            offset: self.offset.resolve(ic, types)?,
        })
    }
}

pub trait ResolveModule {
    fn resolve(self, idents: ModuleIdentifiers) -> Result<Module<Resolved>>;
}

impl ResolveModule for Module<Unresolved> {
    fn resolve(mut self, mi: ModuleIdentifiers) -> Result<Module<Resolved>> {
        let rc = ResolutionContext::new(mi);
        let customs = self.customs;
        let types = &mut self.types;
        let funcs = resolve_all!(self.funcs, rc, types)?;
        let imports = resolve_all!(self.imports, rc, types)?;
        let exports = resolve_all!(self.exports, rc, types)?;
        let globals = resolve_all!(self.globals, rc, types)?;
        let elems = resolve_all!(self.elems, rc, types)?;
        let start = resolve_option!(self.start, rc, types);
        let data = resolve_all!(self.data, rc, types)?;

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
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<TypeUse<Resolved>> {
        validate_inline_typeuse(&self, types, ic)?;
        match self {
            TypeUse::ById(idx) => Ok(TypeUse::ById(idx.resolve(ic, types)?)),
            // TODO: validate that the functiontype matches any existing one
            TypeUse::NamedInline {
                index,
                functiontype,
            } => Ok(TypeUse::NamedInline {
                functiontype,
                index: index.resolve(ic, types)?,
            }),
            TypeUse::AnonymousInline(functiontype) => {
                // Creating a new inline use if a matching type doesn't exist.
                let existing = types
                    .iter()
                    .position(|t| t.functiontype.anonymously_equals(&functiontype));

                let index: Index<Resolved, TypeIndex> = match existing {
                    Some(existing) => Index::unnamed(existing as u32),
                    None => {
                        let newidx = types.len();
                        types.push(TypeField {
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
fn validate_inline_typeuse(
    typeuse: &TypeUse<Unresolved>,
    types: &[TypeField],
    ic: &ResolutionContext,
) -> Result<()> {
    // We only need to check something if the incoming inline typeuse defined a
    // function and an index explicitly.
    let (new_typeidx, new_functiontype) = match (typeuse.index(), typeuse.function_type()) {
        (Some(ti), Some(ft)) if !ti.name().as_bytes().is_empty() => (ti, ft),
        _ => return Ok(()),
    };

    // If a type doesn't exist, then one is (hopefully) being created. That
    // gets checked elsewhere.
    let existing_functiontype = match ic.typeindex(new_typeidx.name()) {
        Some(ei) => &types[ei as usize].functiontype,
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

impl Resolve<FuncField<Resolved>> for FuncField<Unresolved> {
    fn resolve(
        self,
        ic: &ResolutionContext,
        types: &mut Vec<TypeField>,
    ) -> Result<FuncField<Resolved>> {
        validate_inline_typeuse(&self.typeuse, types, ic)?;

        let typeuse = self.typeuse.resolve(ic, types)?;
        let params = get_func_params(&typeuse, types);

        let localindices = params
            .iter()
            .map(|fp| fp.id.or_empty())
            .chain(self.locals.iter().map(|l| l.id.or_empty()))
            .collect();

        let fic = ic.for_func(localindices);

        let body = self.body.resolve(&fic, types)?;

        Ok(FuncField {
            id: self.id,
            exports: self.exports,
            typeuse: TypeUse::ById(typeuse.index().clone()),
            locals: self.locals,
            body,
        })
    }
}
