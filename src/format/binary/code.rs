use {
    super::{
        ensure_consumed::EnsureConsumed,
        error::{BinaryParseError, Result, WithContext},
        values::ReadWasmValues,
    },
    crate::{
        instructions::{instruction_data, op_consts, Operands, BAD_INSTRUCTION},
        syntax::{
            self, Continuation, Expr, FuncField, Id, Instruction, Local, Opcode, Resolved, TypeUse,
        },
        types::ValueType,
    },
    std::io::{Read, Write},
};

#[derive(Debug)]
pub enum ExpressionEnd {
    End,
    Else,
}

#[derive(Debug)]
pub enum InstructionOrEnd {
    End(ExpressionEnd),
    Instruction(Instruction<Resolved>),
}

#[derive(Debug)]
pub struct ExpressionWithEnd {
    expr: Expr<Resolved>,
    end:  ExpressionEnd,
}

const MAX_LOCALS_PER_FUNC: usize = (u32::MAX - 1) as usize;

type LocalCount = u32;

/// Read the Code section of a binary module.
/// codesec := section vec(code)
/// code := size:u32 code:func
/// func := (t*)*:vec(locals) e:expr
/// locals := n:u32 t:type
/// expr := (instr)*
pub trait ReadCode: ReadWasmValues {
    fn read_code_section(&mut self) -> Result<Vec<FuncField<Resolved>>> {
        self.read_vec(|_, s| s.read_func().ctx("reading func"))
    }

    fn read_vec_exprs(&mut self) -> Result<Vec<Expr<Resolved>>> {
        self.read_vec(|_, s| s.read_expr().ctx("reading expr"))
    }

    /// code := size:u32 code:func
    /// func := (t*)*:vec(locals) e:expr
    /// The size is the size in bytes of the entire section, locals + exprs
    fn read_func(&mut self) -> Result<FuncField<Resolved>> {
        let codesize = self.read_u32_leb_128().ctx("parsing func")?;
        let mut code_reader = self.take(codesize as u64);
        let function = FuncField {
            id:      None,
            exports: vec![],
            // The types are parsed earlier and will be set on the returned values.
            typeuse: TypeUse::default(),
            locals:  code_reader.read_locals().ctx("parsing locals")?,
            body:    code_reader.read_expr().ctx("parsing code")?,
        };
        code_reader.ensure_consumed()?;
        Ok(function)
    }

    fn read_local_record(&mut self) -> Result<(LocalCount, ValueType)> {
        Ok((
            self.read_u32_leb_128().ctx("parsing type rep")?,
            self.read_value_type().ctx("parsing value type")?,
        ))
    }

    /// Read the locals description for the function.
    /// locals := n:u32 t:type
    fn read_locals(&mut self) -> Result<Vec<Local>> {
        let local_records = self.read_vec(|_, s| s.read_local_record())?;

        let total: usize = local_records.iter().map(|(cnt, _)| *cnt as usize).sum();

        if total > MAX_LOCALS_PER_FUNC {
            return Err(BinaryParseError::TooManyLocals);
        }

        let result: Vec<Local> = local_records
            .into_iter()
            .flat_map(|(reps, valtype)| {
                std::iter::repeat_with(move || Local { id: None, valtype }).take(reps as usize)
            })
            .collect();

        Ok(result)
    }

    fn read_expr(&mut self) -> Result<Expr<Resolved>> {
        self.read_expr_with_end().map(|ee| ee.expr)
    }

    /// Read the instructions from one function in the code section.
    /// The code is stored in the module as raw bytes, mostly following the
    /// same structure that it has in the binary module ,but with LEB128 numbers
    /// converted to little-endian format.
    /// expr := (instr)* 0x0B
    fn read_expr_with_end(&mut self) -> Result<ExpressionWithEnd> {
        let mut expr = Expr::default();
        let end = loop {
            match self.read_inst()? {
                InstructionOrEnd::Instruction(inst) => expr.instr.push(inst),
                InstructionOrEnd::End(end) => break end,
            }
        };
        Ok(ExpressionWithEnd { expr, end })
    }

    fn read_secondary_opcode(&mut self) -> Result<u8> {
        let secondary = self.read_u32_leb_128().ctx("parsing secondary opcode")?;

        secondary
            .try_into()
            .map_err(|_| BinaryParseError::InvalidSecondaryOpcode(secondary))
    }

    /// Returns -1 if EOF or end instruction was reached while parsing an
    /// opcode. Returns 1 if a new block was started
    /// Returns 0 if a normal instruction was parsed.
    /// Returns Err result otherwise.
    fn read_inst(&mut self) -> Result<InstructionOrEnd> {
        let mut opcode_buf = [0u8; 1];
        self.read_exact(&mut opcode_buf).ctx("parsing opcode")?;

        let opcode = match opcode_buf[0] {
            op_consts::EXTENDED_PREFIX => Opcode::Extended(self.read_secondary_opcode()?),
            op_consts::SIMD_PREFIX => Opcode::Simd(self.read_secondary_opcode()?),
            _ => Opcode::Normal(opcode_buf[0]),
        };

        let instruction_data = match opcode {
            Opcode::Normal(0x0B) => return Ok(InstructionOrEnd::End(ExpressionEnd::End)),
            Opcode::Normal(0x05) => return Ok(InstructionOrEnd::End(ExpressionEnd::Else)),
            _ => instruction_data(&opcode),
        };

        if instruction_data == &BAD_INSTRUCTION {
            return Err(BinaryParseError::InvalidOpcode(opcode));
        }

        // Handle any additional behavior
        let operands = match instruction_data.operands {
            Operands::None => syntax::Operands::None,
            Operands::FuncIndex => syntax::Operands::FuncIndex(self.read_index_use()?),
            Operands::LocalIndex => syntax::Operands::LocalIndex(self.read_index_use()?),
            Operands::GlobalIndex => syntax::Operands::GlobalIndex(self.read_index_use()?),
            Operands::TableIndex => syntax::Operands::TableIndex(self.read_index_use()?),
            Operands::DataIndex => syntax::Operands::DataIndex(self.read_index_use()?),
            Operands::MemoryIndex => syntax::Operands::MemoryIndex(self.read_index_use()?),
            Operands::Br => syntax::Operands::LabelIndex(self.read_index_use()?),
            Operands::BrTable => {
                let mut idxs = self.read_vec(|_, s| s.read_index_use())?;
                let last = self.read_index_use()?;
                idxs.push(last);
                syntax::Operands::BrTable(idxs)
            }
            Operands::I32 => syntax::Operands::I32(self.read_i32_leb_128().ctx("i32")? as u32),
            Operands::I64 => syntax::Operands::I64(self.read_i64_leb_128().ctx("i64")? as u64),
            Operands::F32 => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf).ctx("reading f32 byte")?;
                let val = f32::from_bits(u32::from_le_bytes(buf));
                syntax::Operands::F32(val)
            }
            Operands::F64 => {
                let mut buf = [0u8; 8];
                self.read_exact(&mut buf).ctx("reading f64 byte")?;
                let val = f64::from_bits(u64::from_le_bytes(buf));
                syntax::Operands::F64(val)
            }
            Operands::Memargs => syntax::Operands::Memargs(
                self.read_u32_leb_128().ctx("memarg1")?,
                self.read_u32_leb_128().ctx("memarg2")?,
            ),
            Operands::MemorySize
            | Operands::MemoryGrow
            | Operands::MemoryInit
            | Operands::MemoryFill => {
                self.read_byte()?;
                syntax::Operands::None
            }
            Operands::MemoryCopy => {
                self.read_byte()?;
                self.read_byte()?;
                syntax::Operands::None
            }
            Operands::Block => {
                let bt = self.read_type_use()?;
                let expr = self.read_expr()?;
                syntax::Operands::Block(None, bt, expr, Continuation::End)
            }
            Operands::Loop => {
                let bt = self.read_type_use()?;
                let expr = self.read_expr()?;
                syntax::Operands::Block(None, bt, expr, Continuation::Start)
            }
            Operands::If => {
                let bt = self.read_type_use()?;
                let th = self.read_expr_with_end()?;
                let el = if matches!(th.end, ExpressionEnd::Else) {
                    self.read_expr()?
                } else {
                    Expr::default()
                };
                syntax::Operands::If(None, bt, th.expr, el)
            }
            Operands::HeapType => {
                let ht = self.read_ref_type()?;
                syntax::Operands::HeapType(ht)
            }
            _ => {
                panic!(
                    "unsupported operands {:x?} for {}",
                    instruction_data.operands, opcode
                )
            }
        };

        Ok(InstructionOrEnd::Instruction(Instruction {
            name: Id::default(),
            opcode,
            operands,
        }))
    }

    /// Clarity method: use to read a single LEB128 argument for an instruction.
    fn read_u32_arg(&mut self, out: &mut impl Write) -> Result<()> {
        self.emit_read_u32_leb_128(out).ctx("parsing arg 1/1")
    }

    /// Read one LEB128 value and emit it to the provided writer.
    fn emit_read_u32_leb_128(&mut self, out: &mut impl Write) -> Result<()> {
        out.write(
            &self
                .read_u32_leb_128()
                .ctx("reading leb 128")?
                .to_le_bytes(),
        )
        .ctx("writing leb 128")?;
        Ok(())
    }
}

impl<I: ReadWasmValues> ReadCode for I {}
