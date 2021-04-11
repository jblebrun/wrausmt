use super::{code::ReadCode, values::ReadWasmValues};
use crate::{
    error::{Result, ResultFrom},
    module::{index, Elem, ElemMode},
    types::RefType,
};

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
    /// Read a funcs section. This is just a vec(TypeIndex).
    /// The values here don't correspond to a real module section, instead they
    /// correlate with the rest of the function data in the code section.
    fn read_elems_section(&mut self) -> Result<Box<[Elem]>> {
        self.read_vec(|_, s| {
            let variants = ElemVariant::new(s.read_byte()?);

            let tidx = if variants.has_tableidx() {
                // read table idx
                s.read_u32_leb_128()?
            } else {
                0
            };

            let offset_expr = if variants.active() {
                // read offset expr
                s.read_expr()?
            } else {
                Box::new([])
            };

            let (init_expr, typekind) = if variants.use_initexpr() {
                (
                    s.read_vec_exprs()?,
                    if variants.read_eltypekind() {
                        // read element kind
                        s.read_u32_leb_128().wrap("parsing element kind")?;
                        // Only expect 0 -> funcref for now
                        RefType::Func
                    } else {
                        RefType::Func
                    },
                )
            } else {
                (
                    // read vec(funcidx), generate ref.func expr
                    s.read_vec_funcidx()?
                        .iter()
                        .map(|_| {
                            let genexpr = vec![0xD2u8, 0x00, 0x00, 0x00, 0x00];
                            genexpr.into_boxed_slice()
                        })
                        .collect(),
                    if variants.read_eltypekind() {
                        // read elemnt type
                        s.read_ref_type().wrap("parsing reftype")?
                    } else {
                        RefType::Func
                    },
                )
            };

            let mode = if variants.active() {
                ElemMode::Active {
                    idx: tidx,
                    offset: offset_expr,
                }
            } else if variants.passive() {
                ElemMode::Passive
            } else {
                ElemMode::Declarative
            };

            Ok(Elem {
                typ: typekind,
                init: init_expr,
                mode,
            })
        })
    }

    fn read_vec_funcidx(&mut self) -> Result<Box<[index::Func]>> {
        let items = self.read_u32_leb_128().wrap("parsing item count")?;
        (0..items)
            .map(|_| self.read_u32_leb_128().wrap("reading funcidx"))
            .collect()
    }
}

impl<I: ReadWasmValues + ReadCode> ReadElems for I {}
