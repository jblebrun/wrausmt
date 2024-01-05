use {
    super::{error::Result, leb128::ReadLeb128, BinaryParser, ParserReader},
    crate::{
        binary::error::{BinaryParseErrorKind, ParseResult},
        pctx,
    },
    wrausmt_runtime::syntax::{
        self, types::RefType, ElemField, ElemList, Expr, FuncIndex, Id, Index, Instruction,
        ModeEntry, Opcode, Resolved, TablePosition, TableUse,
    },
};

#[derive(Debug)]
struct ElemVariant {
    bit0: bool,
    bit1: bool,
    bit2: bool,
}

impl ElemVariant {
    fn new(fields: u32) -> Self {
        ElemVariant {
            bit0: (fields & 1) != 0,
            bit1: (fields & 2) != 0,
            bit2: (fields & 4) != 0,
        }
    }

    fn active(&self) -> bool {
        !self.bit0
    }

    fn passive(&self) -> bool {
        self.bit0 && !self.bit1
    }

    fn has_tableidx(&self) -> bool {
        !self.bit0 && self.bit1
    }

    fn use_initexpr(&self) -> bool {
        self.bit2
    }

    fn read_eltypekind(&self) -> bool {
        self.bit0 || self.bit1
    }
}

/// Read the tables section of a binary module from a std::io::Read.
impl<R: ParserReader> BinaryParser<R> {
    pub fn read_elems_section(&mut self) -> Result<Vec<ElemField<Resolved>>> {
        pctx!(self, "read elems section");
        self.read_vec(|_, s| s.read_elem())
    }

    fn read_elem_kind(&mut self) -> Result<RefType> {
        pctx!(self, "read elem kind");
        // read elemkind type, always 0
        let elemkind = self.read_byte()?;
        if elemkind != 0 {
            return Err(self.err(BinaryParseErrorKind::InvalidElemKind(elemkind)));
        }
        Ok(RefType::Func)
    }

    fn read_init_funcs(&mut self) -> Result<Vec<Expr<Resolved>>> {
        pctx!(self, "read init funcs");
        Ok(self
            .read_vec_funcidx()?
            .into_iter()
            .map(|idx| Expr {
                instr: vec![Instruction {
                    name:     Id::literal("ref.func"),
                    opcode:   Opcode::Normal(0xD2),
                    operands: syntax::Operands::FuncIndex(idx),
                }],
            })
            .collect())
    }

    fn read_elem(&mut self) -> Result<ElemField<Resolved>> {
        pctx!(self, "read elem");
        let variants = ElemVariant::new(self.read_u32_leb_128().result(self)?);

        let tidx = if variants.has_tableidx() {
            // read table idx
            self.read_index_use()?
        } else {
            Index::default()
        };

        let offset_expr = if variants.active() {
            // read offset expr
            self.read_expr()?
        } else {
            Expr::default()
        };

        let (typekind, init_expr) = if variants.use_initexpr() {
            let reftype = if variants.read_eltypekind() {
                // read element kind
                self.read_ref_type()?
            } else {
                RefType::Func
            };
            (reftype, self.read_vec_exprs()?)
        } else {
            let elemkind = if variants.read_eltypekind() {
                self.read_elem_kind()?
            } else {
                RefType::Func
            };
            // read vec(funcidx), generate ref.func expr
            let init_exprs = self.read_init_funcs()?;
            (elemkind, init_exprs)
        };

        let mode = if variants.active() {
            let tableuse = TableUse { tableidx: tidx };
            ModeEntry::Active(TablePosition {
                tableuse,
                offset: offset_expr,
            })
        } else if variants.passive() {
            ModeEntry::Passive
        } else {
            ModeEntry::Declarative
        };

        let elemlist = ElemList {
            reftype: typekind,
            items:   init_expr,
        };

        Ok(ElemField {
            id: None,
            mode,
            elemlist,
        })
    }

    fn read_vec_funcidx(&mut self) -> Result<Vec<Index<Resolved, FuncIndex>>> {
        pctx!(self, "read vec funcidx");
        let items = self.read_u32_leb_128().result(self)?;
        (0..items).map(|_| self.read_index_use()).collect()
    }
}
