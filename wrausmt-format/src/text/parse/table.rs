use {
    super::{error::Result, module::Field, pctx, Parser},
    std::io::Read,
    wrausmt_runtime::syntax::{
        types::{RefType, TableType},
        ElemField, ElemList, Expr, ImportDesc, ImportField, Instruction, ModeEntry, TableField,
        TablePosition, TableUse, Unresolved,
    },
};

impl<R: Read> Parser<R> {
    fn try_index_as_funcref(&mut self) -> Result<Option<Expr<Unresolved>>> {
        pctx!(self, "try index as funcref");
        Ok(self.try_index()?.map(|idx| Expr {
            instr: vec![Instruction::reffunc(idx)],
        }))
    }

    fn read_table_inline_func_elems(&mut self) -> Result<ElemList<Unresolved>> {
        pctx!(self, "read table inline func elems");
        // ‘(’ ‘𝚝𝚊𝚋𝚕𝚎’  𝚒𝚍?  𝚛𝚎𝚏𝚝𝚢𝚙𝚎  ‘(’ ‘𝚎𝚕𝚎𝚖’  𝚎𝚕𝚎𝚖𝚕𝚒𝚜𝚝 ‘)’
        // ‘(’ ‘𝚝𝚊𝚋𝚕𝚎’  𝚒𝚍?  𝚛𝚎𝚏𝚝𝚢𝚙𝚎  ‘(’ ‘𝚎𝚕𝚎𝚖’  𝑥𝑛:𝚟𝚎𝚌(𝚎𝚡𝚙𝚛) ‘)’  ‘)’
        self.expect_expr_start("elem")?;

        // FuncRefs may just be a vector of indices
        let mut items = self.zero_or_more(Self::try_index_as_funcref)?;
        if items.is_empty() {
            items = self.zero_or_more(Self::try_item_expression)?;
        }

        self.expect_close()?;
        Ok(ElemList::func(items))
    }

    pub fn try_table_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        pctx!(self, "try table field");
        if !self.try_expr_start("table")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        let exports = self.zero_or_more(Self::try_inline_export)?;

        let import = self.try_inline_import()?;

        if let Some(import) = import {
            let tabletype = self.expect_tabletype()?;
            self.expect_close()?;
            return Ok(Some(Field::Import(ImportField {
                id,
                modname: import.0,
                name: import.1,
                exports,
                desc: ImportDesc::Table(tabletype),
            })));
        }

        let (tabletype, elemfield) = match self.try_reftype()? {
            Some(RefType::Func) => {
                let elemlist = self.read_table_inline_func_elems()?;
                let tabletype = TableType::fixed_size(elemlist.items.len() as u32);
                let elemfied = Some(ElemField {
                    id: None,
                    mode: ModeEntry::Active(TablePosition::default()),
                    elemlist,
                });
                (tabletype, elemfied)
            }
            _ => (self.expect_tabletype()?, None),
        };

        self.expect_close()?;

        Ok(Some(Field::Table(
            TableField {
                id,
                exports,
                tabletype,
            },
            elemfield,
        )))
    }

    fn try_table_use(&mut self) -> Result<Option<TableUse<Unresolved>>> {
        pctx!(self, "try table use");
        if !self.try_expr_start("table")? {
            return Ok(None);
        }
        let tableidx = self.expect_index()?;
        self.expect_close()?;
        Ok(Some(TableUse { tableidx }))
    }

    // <reftype> vec<elemexpr>
    // elemexpr := ('item' <expr>)
    //           | <instr>
    //           | func <index>*
    fn try_elemlist(&mut self, allow_bare_funcidx: bool) -> Result<ElemList<Unresolved>> {
        pctx!(self, "try elemlist");
        let _reftype = self.try_reftype()?;
        let items = self.zero_or_more(Self::try_item_expression)?;
        if !items.is_empty() {
            return Ok(ElemList::func(items));
        }

        if self.take_keyword_if(|kw| kw == "func")?.is_some() || allow_bare_funcidx {
            let items = self.zero_or_more(Self::try_index_as_funcref)?;
            return Ok(ElemList::func(items));
        }

        Ok(ElemList::func(vec![]))
    }

    // (elem <id>? <elemlist>) -> passive
    // (elem <id>? <tableuse> (offset <expr>) <elemlist>) -> active
    // (elem <id>? declare <elemlist>) -> declarative
    // <tableuse> := (table <idx>)
    // (<instr>) === (offset <instr>)
    // (<instr>) === (item <instr>)
    // tableuse can be omitted, defaulting to 0.
    pub fn try_elem_field(&mut self) -> Result<Option<Field<Unresolved>>> {
        pctx!(self, "try elem field");
        if !self.try_expr_start("elem")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        let tableuse = self.try_table_use()?;

        let declarative = self.take_keyword_if(|kw| kw == "declare")?;

        let offset = self.try_offset_expression()?;

        let elemlist = self.try_elemlist(tableuse.is_none())?;

        let mode = if declarative.is_some() {
            ModeEntry::Declarative
        } else if let Some(offset) = offset {
            let tableuse = tableuse.unwrap_or_else(TableUse::default);
            ModeEntry::Active(TablePosition { tableuse, offset })
        } else {
            ModeEntry::Passive
        };
        self.expect_close()?;
        Ok(Some(Field::Elem(ElemField { id, mode, elemlist })))
    }
}
