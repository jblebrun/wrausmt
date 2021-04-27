use super::syntax::{
    DataField, ElemField, ExportDesc, ExportField, FuncField, GlobalField, ImportDesc, ImportField,
    Index, MemoryField, Module, StartField, TableField, TypeField
};

pub struct ModuleBuilder {
    module: Module

}

impl ModuleBuilder {
    pub fn new(id: Option<String>) -> Self {
        Self {
            module: Module {
                id, 
                ..Module::default()
            }
        }
    }

    pub fn build(self) -> Module {
        self.module
    }

    pub fn add_typefield(&mut self, typefield: TypeField) {
        self.module.types.push(typefield);
    }

    pub fn add_funcfield(&mut self, f: FuncField) {
        // type use may define new type
        if let Some(inline_typefield) = f.typeuse.get_inline_def() {
            if !self.module.types.contains(&inline_typefield) {
                self.module.types.push(inline_typefield);
            }
        }
        // export field may define new exports.
        let funcidx = self.module.funcs.len() as u32;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name: export_name.clone(),
                exportdesc: ExportDesc::Func(Index::Numeric(funcidx)),
            })
        }
        self.module.funcs.push(f);
    }

    pub fn add_tablefield(&mut self, f: TableField) {
        // export field may define new exports.
        let tableidx = self.module.funcs.len() as u32;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name: export_name.clone(),
                exportdesc: ExportDesc::Table(Index::Numeric(tableidx)),
            })
        }
        // TODO elem contents may define new elem
        self.module.tables.push(f);
    }

    pub fn add_memoryfield(&mut self, f: MemoryField) {
        // export field may define new exports.
        let memidx = self.module.funcs.len() as u32;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name: export_name.clone(),
                exportdesc: ExportDesc::Mem(Index::Numeric(memidx)),
            })
        }
        self.module.memories.push(f);
        // data contents may define new data
    }

    pub fn add_importfield(&mut self, f: ImportField) {
        // Function import may define a new type.
        if let ImportDesc::Func(tu) = &f.desc {
            if let Some(inline_typefield) = tu.get_inline_def() {
                if !self.module.types.contains(&inline_typefield) {
                    self.module.types.push(inline_typefield);
                }
            }
        }
        self.module.imports.push(f);
    }

    pub fn add_exportfield(&mut self, f: ExportField) {
        self.module.exports.push(f)
    }

    pub fn add_globalfield(&mut self, f: GlobalField) {
        let globalidx = self.module.funcs.len() as u32;
        for export_name in &f.exports {
            self.module.exports.push(ExportField {
                name: export_name.clone(),
                exportdesc: ExportDesc::Mem(Index::Numeric(globalidx))
            })
        }
        // export field may define new exports.
        self.module.globals.push(f);
    }

    pub fn add_startfield(&mut self, f: StartField) {
        self.module.start = Some(f)
    }

    pub fn add_elemfield(&mut self, f: ElemField) {
        self.module.elems.push(f)
    }

    pub fn add_datafield(&mut self, f: DataField) {
        self.module.data.push(f) 
    }
}
