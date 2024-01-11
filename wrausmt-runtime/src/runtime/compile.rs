use crate::{
    instructions::op_consts,
    runtime::instance::{ExportInstance, ExternalVal, ModuleInstance},
    syntax::{
        self,
        types::{FunctionType, RefType, ValueType},
        FuncField, Instruction, Opcode, Resolved, TypeUse, UncompiledExpr,
    },
    validation::{Result, Validation, ValidationContext, ValidationMode},
};

const END_OPCODE: Opcode = Opcode::Normal(0xb);
const ELSE_OPCODE: Opcode = Opcode::Normal(0x5);

impl From<syntax::FunctionType> for FunctionType {
    fn from(ast: syntax::FunctionType) -> FunctionType {
        FunctionType {
            params: ast.params.iter().map(|p| p.valuetype).collect(),
            result: ast.results.iter().map(|r| r.valuetype).collect(),
        }
    }
}

impl From<syntax::Local> for ValueType {
    fn from(ast: syntax::Local) -> ValueType {
        ast.valtype
    }
}

fn compile_export_desc(ast: syntax::ExportDesc<Resolved>, modinst: &ModuleInstance) -> ExternalVal {
    match ast {
        syntax::ExportDesc::Func(idx) => ExternalVal::Func(modinst.func(idx.value())),
        syntax::ExportDesc::Table(idx) => ExternalVal::Table(modinst.table(idx.value())),
        syntax::ExportDesc::Mem(idx) => ExternalVal::Memory(modinst.mem(idx.value())),
        syntax::ExportDesc::Global(idx) => ExternalVal::Global(modinst.global(idx.value())),
    }
}

pub fn compile_export(
    ast: syntax::ExportField<Resolved>,
    modinst: &ModuleInstance,
) -> ExportInstance {
    ExportInstance {
        name: ast.name,
        addr: compile_export_desc(ast.exportdesc, modinst),
    }
}

pub trait Emitter {
    fn validate_instr(&mut self, instr: &Instruction<Resolved>) -> crate::validation::Result<()>;
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
        self.emit_opcode(END_OPCODE);

        // If the continuation is at the end of the block, we go back to the space
        // we reserved (just above), and write self the position of the last item
        // in the current compiled selfput, which corresponds to the end of the
        // expression.
        if matches!(cnt, syntax::Continuation::End) {
            self.splice32(continuation_location, self.len() as u32);
        }
        Ok(())
    }

    fn emit_br_table(&mut self, indices: &[syntax::Index<Resolved, syntax::LabelIndex>]) {
        self.emit32(indices.len() as u32);
        for i in indices {
            self.emit32(i.value());
        }
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
            self.emit_opcode(ELSE_OPCODE);
            self.emit_expr(el)?;
        } else {
            self.splice32(else_location, self.len() as u32);
        }

        self.emit_opcode(END_OPCODE);
        self.splice32(end_location, self.len() as u32);
        Ok(())
    }

    fn emit_expr(&mut self, expr: &syntax::UncompiledExpr<Resolved>) -> Result<()> {
        for instr in &expr.instr {
            self.validate_instr(instr)?;

            // Emit opcode
            self.emit_opcode(instr.opcode);

            // Emit operands
            match &instr.operands {
                syntax::Operands::None => (),
                syntax::Operands::Block(_, typeuse, e, cnt) => self.emit_block(typeuse, e, cnt)?,
                syntax::Operands::BrTable(indices) => self.emit_br_table(indices),
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
        }
        Ok(())
    }
}

struct Compiler<'a> {
    output:     Vec<u8>,
    validation: Validation<'a>,
}
impl<'a> Compiler<'a> {
    pub fn new(
        validation_mode: ValidationMode,
        modinst: &'a ModuleInstance,
        localtypes: &'a [ValueType],
        resulttypes: &'a [ValueType],
    ) -> Compiler<'a> {
        Compiler {
            output:     Vec::new(),
            validation: Validation::new(ValidationContext::new(
                validation_mode,
                modinst,
                localtypes,
                resulttypes,
            )),
        }
    }

    pub fn finish(self) -> Result<Box<[u8]>> {
        self.validation.finish()?;
        Ok(self.output.into_boxed_slice())
    }
}

impl<'a> Emitter for Compiler<'a> {
    fn validate_instr(&mut self, instr: &Instruction<Resolved>) -> crate::validation::Result<()> {
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

/// Compile the body of the provided [`FuncField`] as if it were the provided
/// type. Instructions will be validated using the provided [`ValidationMode`].
/// Validation uses the provided [`ModuleInstance`] to resolve module-wide
/// indices.
pub fn compile_function_body(
    validation_mode: ValidationMode,
    func: &FuncField<Resolved, UncompiledExpr<Resolved>>,
    functype: &FunctionType,
    modinst: &ModuleInstance,
) -> Result<Box<[u8]>> {
    let mut localtypes = functype.params.to_vec();
    localtypes.extend(func.locals.iter().map(|l| l.valtype));

    let mut out = Compiler::new(validation_mode, modinst, &localtypes, &functype.result);

    out.emit_expr(&func.body)?;
    out.emit_opcode(END_OPCODE);

    out.finish()
}

/// Compile the body of the provided [`FuncField`] as if it were the provided
/// type. Instructions will be validated using the provided [`ValidationMode`].
/// Validation uses the provided [`ModuleInstance`] to resolve module-wide
/// indices. A final `END` opcode will not be emitted.
pub fn compile_simple_expression(
    validation_mode: ValidationMode,
    expr: &UncompiledExpr<Resolved>,
    modinst: &ModuleInstance,
) -> Result<Box<[u8]>> {
    let mut out = Compiler::new(validation_mode, modinst, &[], &[]);
    out.emit_expr(expr)?;

    out.finish()
}
