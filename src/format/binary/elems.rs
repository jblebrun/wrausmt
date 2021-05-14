use super::{code::ReadCode, values::ReadWasmValues};
use crate::{
    err,
    error::{Result, ResultFrom},
    syntax::{
        self, ElemField, ElemList, Expr, FuncIndex, Index, Instruction, ModeEntry, Resolved,
        TablePosition, TableUse,
    },
    types::RefType,
};

#[derive(Debug)]
struct ElemVariant {
    bit0: bool,
    bit1: bool,
    bit2: bool,
}

impl ElemVariant {
    fn new(fields: u8) -> Self {
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
pub trait ReadElems: ReadWasmValues + ReadCode {
    fn read_elems_section(&mut self) -> Result<Vec<ElemField<Resolved>>> {
        self.read_vec(|_, s| s.read_elem())
    }

    fn read_elem(&mut self) -> Result<ElemField<Resolved>> {
        let variants = ElemVariant::new(self.read_byte()?);

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

        let (init_expr, typekind) = if variants.use_initexpr() {
            (
                self.read_vec_exprs()?,
                if variants.read_eltypekind() {
                    // read element kind
                    self.read_u32_leb_128().wrap("parsing element kind")?;
                    // Only expect 0 -> funcref for now
                    RefType::Func
                } else {
                    RefType::Func
                },
            )
        } else {
            (
                // read vec(funcidx), generate ref.func expr
                self.read_vec_funcidx()?
                    .into_iter()
                    .map(|idx| Expr {
                        instr: vec![Instruction {
                            name: "ref.func".to_owned(),
                            opcode: 0xD2,
                            operands: syntax::Operands::FuncIndex(idx),
                        }],
                    })
                    .collect(),
                if variants.read_eltypekind() {
                    // read elemkind type, always 0
                    let elemkind = self.read_byte()?;
                    if elemkind != 0 {
                        return err!("wrong elemkind byte {}", elemkind);
                    }
                    RefType::Func
                } else {
                    RefType::Func
                },
            )
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
            items: init_expr,
        };

        Ok(ElemField {
            id: None,
            mode,
            elemlist,
        })
    }

    fn read_vec_funcidx(&mut self) -> Result<Vec<Index<Resolved, FuncIndex>>> {
        let items = self.read_u32_leb_128().wrap("parsing item count")?;
        (0..items)
            .map(|_| self.read_index_use().wrap("reading funcidx"))
            .collect()
    }
}

impl<I: ReadWasmValues + ReadCode> ReadElems for I {}
