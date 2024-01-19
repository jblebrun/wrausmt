use {
    super::{
        validation::{Result, Validation, ValidationMode},
        ValueTypes,
    },
    wrausmt_runtime::{
        instructions::{op_consts, opcodes},
        syntax::{
            self,
            types::{RefType, ValueType},
            CompiledExpr, FuncField, Id, Instruction, Module, Opcode, Operands, Resolved, TypeUse,
            UncompiledExpr,
        },
    },
};

pub trait Emitter {
    fn validate_instr(&mut self, instr: &Instruction<Resolved>) -> Result<()>;
    fn emit8(&mut self, v: u8);
    fn emit32(&mut self, v: u32);
    fn emit64(&mut self, v: u64);
    fn splice8(&mut self, idx: usize, v: u8);
    fn splice32(&mut self, idx: usize, v: u32);
    fn len(&self) -> usize;
    fn emit_opcode(&mut self, opcode: Opcode);

    fn is_empty(&self) -> bool;

    fn emit_block(
        &mut self,
        typeuse: &syntax::TypeUse<Resolved>,
        expr: &syntax::UncompiledExpr<Resolved>,
        cnt: &syntax::Continuation,
    ) -> Result<()> {
        let startcnt = self.len() as u32 - 1;

        self.emit32(typeuse.index().value());

        let continuation_location = self.len();
        // If the continuation is at the start, we write self the current length
        // of the compiled selfput so far, adding 4 to account for the 4-byte value
        // that we are abself to write.
        if matches!(cnt, syntax::Continuation::Start) {
            self.emit32(startcnt);
        } else {
            // Remember the spot to write continuation, and reserve a spot.
            self.emit32(0x00);
        }

        self.emit_expr(expr)?;
        self.emit_end()?;

        // If the continuation is at the end of the block, we go back to the space
        // we reserved (just above), and write self the position of the last item
        // in the current compiled selfput, which corresponds to the end of the
        // expression.
        if matches!(cnt, syntax::Continuation::End) {
            self.splice32(continuation_location, self.len() as u32);
        }
        Ok(())
    }

    fn emit_br_table(
        &mut self,
        indices: &[syntax::Index<Resolved, syntax::LabelIndex>],
        last: &syntax::Index<Resolved, syntax::LabelIndex>,
    ) {
        self.emit32(indices.len() as u32);
        for i in indices {
            self.emit32(i.value());
        }
        self.emit32(last.value())
    }

    fn emit_if(
        &mut self,
        typeuse: &TypeUse<Resolved>,
        th: &UncompiledExpr<Resolved>,
        el: &UncompiledExpr<Resolved>,
    ) -> Result<()> {
        self.emit32(typeuse.index().value());

        // Store the space for end continuation
        let end_location = self.len();
        self.emit32(0x00);

        // Store the space for else continuation
        let else_location = self.len();
        self.emit32(0x00);

        self.emit_expr(th)?;

        if !el.instr.is_empty() {
            self.splice32(else_location, self.len() as u32 + 1);
            // Replace the `end` for the then expression with the else opcode.
            self.emit_else()?;
            self.emit_expr(el)?;
        } else {
            self.splice32(else_location, self.len() as u32);
        }

        self.emit_end()?;
        self.splice32(end_location, self.len() as u32);
        Ok(())
    }

    fn emit_expr(&mut self, expr: &syntax::UncompiledExpr<Resolved>) -> Result<()> {
        expr.instr.iter().try_for_each(|i| self.emit_instr(i))
    }

    fn emit_instr(&mut self, instr: &Instruction<Resolved>) -> Result<()> {
        self.validate_instr(instr)?;

        // Emit opcode
        self.emit_opcode(instr.opcode);

        // Emit operands
        match &instr.operands {
            syntax::Operands::None => (),
            syntax::Operands::Block(_, typeuse, e, cnt) => self.emit_block(typeuse, e, cnt)?,
            syntax::Operands::BrTable(indices, last) => self.emit_br_table(indices, last),
            syntax::Operands::CallIndirect(idx, typeuse) => {
                self.emit32(idx.value());
                self.emit32(typeuse.index().value());
            }
            syntax::Operands::Select(_) => (),
            syntax::Operands::If(_, typeuse, th, el) => self.emit_if(typeuse, th, el)?,
            syntax::Operands::I32(n) => self.emit32(*n),
            syntax::Operands::I64(n) => self.emit64(*n),
            syntax::Operands::F32(n) => self.emit32(n.to_bits()),
            syntax::Operands::F64(n) => self.emit64(n.to_bits()),
            syntax::Operands::FuncIndex(idx) => self.emit32(idx.value()),
            syntax::Operands::TableIndex(idx) => self.emit32(idx.value()),
            syntax::Operands::GlobalIndex(idx) => self.emit32(idx.value()),
            syntax::Operands::ElemIndex(idx) => self.emit32(idx.value()),
            syntax::Operands::DataIndex(idx) => self.emit32(idx.value()),
            syntax::Operands::LocalIndex(idx) => self.emit32(idx.value()),
            syntax::Operands::LabelIndex(idx) => self.emit32(idx.value()),
            syntax::Operands::MemoryIndex(idx) => self.emit32(idx.value()),
            syntax::Operands::Memargs1(o, a)
            | syntax::Operands::Memargs2(o, a)
            | syntax::Operands::Memargs4(o, a)
            | syntax::Operands::Memargs8(o, a) => {
                self.emit32(*o);
                self.emit32(*a)
            }
            syntax::Operands::TableInit(ti, ei) => {
                self.emit32(ti.value());
                self.emit32(ei.value());
            }
            syntax::Operands::TableCopy(ti, t2i) => {
                self.emit32(ti.value());
                self.emit32(t2i.value());
            }
            syntax::Operands::HeapType(ht) => {
                // Use the binary format encoding of ref type.
                let htbyte = match ht {
                    RefType::Func => 0x70,
                    RefType::Extern => 0x6F,
                };
                self.emit8(htbyte);
            }
        }
        Ok(())
    }

    fn emit_end(&mut self) -> Result<()> {
        self.emit_instr(&Instruction {
            name:     Id::literal("end"),
            opcode:   opcodes::END,
            operands: Operands::None,
        })
    }

    fn emit_else(&mut self) -> Result<()> {
        self.emit_instr(&Instruction {
            name:     Id::literal("else"),
            opcode:   opcodes::ELSE,
            operands: Operands::None,
        })
    }
}

pub struct ValidatingEmitter<'a> {
    output:     Vec<u8>,
    validation: Validation<'a>,
}
impl<'a> ValidatingEmitter<'a> {
    /// Compile a Function's body. Instructions will be validated using the
    /// provided [`ValidationMode`]. Validation uses the provided
    /// [`Module`] to resolve module-wide indices.
    pub fn function_body(
        validation_mode: ValidationMode,
        module: &Module<Resolved, UncompiledExpr<Resolved>>,
        func: &FuncField<Resolved, UncompiledExpr<Resolved>>,
    ) -> Result<CompiledExpr> {
        let functype = &module.types[func.typeuse.index().value() as usize].functiontype;

        let mut localtypes = functype.params.valuetypes();
        localtypes.extend(func.locals.iter().map(|l| l.valtype));

        let resulttypes: Vec<_> = functype.results.valuetypes();

        let mut out = ValidatingEmitter::new(validation_mode, module, localtypes, resulttypes);

        out.emit_expr(&func.body)?;
        out.emit_end()?;

        Ok(CompiledExpr {
            instr: out.finish()?,
        })
    }

    /// Compile the body of the provided [`FuncField`] as if it were the
    /// provided type. Instructions will be validated using the provided
    /// [`ValidationMode`]. Validation uses the provided [`Module`] to
    /// resolve module-wide indices. A final `END` opcode will not be
    /// emitted.
    pub fn simple_expression(
        validation_mode: ValidationMode,
        module: &Module<Resolved, UncompiledExpr<Resolved>>,
        expr: &UncompiledExpr<Resolved>,
    ) -> Result<CompiledExpr> {
        Self::simple_expressions(validation_mode, module, &[expr])
    }

    pub fn simple_expressions(
        validation_mode: ValidationMode,
        module: &Module<Resolved, UncompiledExpr<Resolved>>,
        exprs: &[&UncompiledExpr<Resolved>],
    ) -> Result<CompiledExpr> {
        let mut out = ValidatingEmitter::new(validation_mode, module, vec![], vec![]);
        for expr in exprs {
            out.emit_expr(expr)?;
        }
        Ok(CompiledExpr {
            instr: out.finish()?,
        })
    }

    fn new(
        validation_mode: ValidationMode,
        module: &'a Module<Resolved, UncompiledExpr<Resolved>>,
        localtypes: Vec<ValueType>,
        resulttypes: Vec<ValueType>,
    ) -> ValidatingEmitter<'a> {
        ValidatingEmitter {
            output:     Vec::new(),
            validation: Validation::new(validation_mode, module, localtypes, resulttypes),
        }
    }

    fn finish(self) -> Result<Box<[u8]>> {
        Ok(self.output.into_boxed_slice())
    }
}

impl<'a> Emitter for ValidatingEmitter<'a> {
    fn validate_instr(&mut self, instr: &Instruction<Resolved>) -> Result<()> {
        self.validation.handle_instr(instr)
    }

    fn emit8(&mut self, v: u8) {
        self.output.push(v);
    }

    fn emit32(&mut self, v: u32) {
        let bytes = &v.to_le_bytes()[..];
        self.output.extend(bytes);
    }

    fn splice32(&mut self, idx: usize, v: u32) {
        let bytes = &v.to_le_bytes();
        self.output.splice(idx..idx + 4, bytes.iter().cloned());
    }

    fn splice8(&mut self, idx: usize, v: u8) {
        self.output[idx] = v;
    }

    fn emit64(&mut self, v: u64) {
        let bytes = &v.to_le_bytes()[..];
        self.output.extend(bytes);
    }

    fn emit_opcode(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::Normal(o) => self.output.push(o),
            Opcode::Extended(o) => self.output.extend(&[op_consts::EXTENDED_PREFIX, o]),
            Opcode::Simd(o) => self.output.extend(&[op_consts::SIMD_PREFIX, o]),
        }
    }

    fn len(&self) -> usize {
        self.output.len()
    }

    fn is_empty(&self) -> bool {
        self.output.is_empty()
    }
}
