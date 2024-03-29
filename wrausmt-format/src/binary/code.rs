use {
    super::{
        error::{BinaryParseErrorKind, Result},
        leb128::ReadLeb128,
        BinaryParser, ParserReader,
    },
    crate::{
        binary::{error::ParseResult, read_with_location::Locate},
        pctx,
    },
    std::io::Read,
    wrausmt_common::true_or::TrueOr,
    wrausmt_runtime::{
        instructions::{instruction_data, op_consts, Operands, BAD_INSTRUCTION},
        syntax::{
            self, types::ValueType, Continuation, FResult, FuncField, Id, Index, Instruction,
            Local, Opcode, Resolved, TypeIndex, TypeUse, UncompiledExpr,
        },
    },
};

#[derive(Debug)]
enum ExpressionEnd {
    End,
    Else,
}

#[derive(Debug)]
enum InstructionOrEnd {
    End(ExpressionEnd),
    Instruction(Instruction<Resolved>),
}

#[derive(Debug)]
pub struct ExpressionWithEnd {
    expr: UncompiledExpr<Resolved>,
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
impl<R: ParserReader> BinaryParser<R> {
    pub(in crate::binary) fn read_code_section(
        &mut self,
        data_indices_ok: bool,
        types: &[Index<Resolved, TypeIndex>],
    ) -> Result<Vec<FuncField<Resolved, UncompiledExpr<Resolved>>>> {
        pctx!(self, "read code section");
        self.read_vec(|i, s| {
            s.read_func(
                data_indices_ok,
                types
                    .get(i as usize)
                    .ok_or(s.err(BinaryParseErrorKind::FuncSizeMismatch))?,
            )
        })
    }

    pub(in crate::binary) fn read_vec_exprs(
        &mut self,
        data_indices_ok: bool,
    ) -> Result<Vec<UncompiledExpr<Resolved>>> {
        pctx!(self, "read vec exprs");
        self.read_vec(|_, s| s.read_expr(data_indices_ok))
    }

    pub(in crate::binary) fn read_expr(
        &mut self,
        data_indices_ok: bool,
    ) -> Result<UncompiledExpr<Resolved>> {
        pctx!(self, "read expr");
        self.read_expr_with_end(data_indices_ok).map(|ee| ee.expr)
    }

    /// code := size:u32 code:func
    /// func := (t*)*:vec(locals) e:expr
    /// The size is the size in bytes of the entire section, locals + exprs
    fn read_func(
        &mut self,
        data_indices_ok: bool,
        tyidx: &Index<Resolved, TypeIndex>,
    ) -> Result<FuncField<Resolved, UncompiledExpr<Resolved>>> {
        pctx!(self, "read func");
        let location = self.location();
        let code_size_expected = self.read_u32_leb_128().result(self)?;

        let (function, amount_read) = self.count_reads(|s| {
            Ok(FuncField {
                id: None,
                exports: vec![],
                // The types are parsed earlier and will be set on the returned values.
                typeuse: TypeUse::ByIndex(tyidx.clone()),
                locals: s.read_locals()?,
                body: s.read_expr(data_indices_ok)?,
                location,
            })
        })?;

        match amount_read {
            cnt if cnt < code_size_expected => Err(self.err(BinaryParseErrorKind::CodeTooShort)),
            cnt if cnt > code_size_expected => Err(self.err(BinaryParseErrorKind::CodeTooLong)),
            _ => Ok(function),
        }
    }

    fn read_local_record(&mut self) -> Result<(LocalCount, ValueType)> {
        pctx!(self, "read local record");
        Ok((
            self.read_u32_leb_128().result(self)?,
            self.read_value_type()?,
        ))
    }

    /// Read the locals description for the function.
    /// locals := n:u32 t:type
    fn read_locals(&mut self) -> Result<Vec<Local>> {
        pctx!(self, "read locals");
        let local_records = self.read_vec(|_, s| s.read_local_record())?;

        let total: usize = local_records.iter().map(|(cnt, _)| *cnt as usize).sum();

        (total <= MAX_LOCALS_PER_FUNC)
            .true_or_else(|| self.err(BinaryParseErrorKind::TooManyLocals))?;

        let result: Vec<Local> = local_records
            .into_iter()
            .flat_map(|(reps, valtype)| {
                std::iter::repeat_with(move || Local { id: None, valtype }).take(reps as usize)
            })
            .collect();

        Ok(result)
    }

    /// Read the instructions from one function in the code section.
    /// The code is stored in the module as raw bytes, mostly following the
    /// same structure that it has in the binary module ,but with LEB128 numbers
    /// converted to little-endian format.
    /// expr := (instr)* 0x0B
    fn read_expr_with_end(&mut self, data_indices_ok: bool) -> Result<ExpressionWithEnd> {
        pctx!(self, "read expr with end");
        let mut instr: Vec<Instruction<Resolved>> = Vec::new();
        let end = loop {
            let inst = self.read_inst(data_indices_ok);
            match inst? {
                InstructionOrEnd::Instruction(inst) => instr.push(inst),
                InstructionOrEnd::End(end) => break end,
            }
        };
        Ok(ExpressionWithEnd {
            expr: UncompiledExpr { instr },
            end,
        })
    }

    fn read_secondary_opcode(&mut self) -> Result<u8> {
        pctx!(self, "read secondary opcode");
        let secondary = self.read_u32_leb_128().result(self)?;

        secondary
            .try_into()
            .map_err(|_| self.err(BinaryParseErrorKind::InvalidSecondaryOpcode(secondary)))
    }

    fn read_zero_byte(&mut self) -> Result<()> {
        let byte = self.read_byte()?;
        (byte == 0).true_or_else(|| self.err(BinaryParseErrorKind::ZeroByteExpected))
    }

    /// Returns -1 if EOF or end instruction was reached while parsing an
    /// opcode. Returns 1 if a new block was started
    /// Returns 0 if a normal instruction was parsed.
    /// Returns Err result otherwise.
    fn read_inst(&mut self, data_indices_ok: bool) -> Result<InstructionOrEnd> {
        pctx!(self, "read inst");
        let location = self.reader.location();
        let mut opcode_buf = [0u8; 1];
        self.read_exact(&mut opcode_buf).result(self)?;

        pctx!(self, &format!("read inst {:?}", opcode_buf));
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

        (instruction_data != &BAD_INSTRUCTION)
            .true_or_else(|| self.err(BinaryParseErrorKind::InvalidOpcode(opcode)))?;

        // Handle any additional behavior
        let operands = match instruction_data.operands {
            Operands::Select | Operands::None => syntax::Operands::None,
            Operands::FuncIndex => syntax::Operands::FuncIndex(self.read_index_use()?),
            Operands::LocalIndex => syntax::Operands::LocalIndex(self.read_index_use()?),
            Operands::GlobalIndex => syntax::Operands::GlobalIndex(self.read_index_use()?),
            Operands::TableIndex => syntax::Operands::TableIndex(self.read_index_use()?),
            Operands::ElemIndex => syntax::Operands::ElemIndex(self.read_index_use()?),
            Operands::DataIndex => {
                data_indices_ok
                    .true_or_else(|| self.err(BinaryParseErrorKind::DataCountMissing))?;
                syntax::Operands::DataIndex(self.read_index_use()?)
            }
            Operands::MemoryIndex => syntax::Operands::MemoryIndex(self.read_index_use()?),
            Operands::TableCopy => {
                let dsttabidx = self.read_index_use()?;
                let srctabidx = self.read_index_use()?;
                syntax::Operands::TableCopy(dsttabidx, srctabidx)
            }
            Operands::TableInit => {
                let elidx = self.read_index_use()?;
                let tabidx = self.read_index_use()?;
                syntax::Operands::TableInit(tabidx, elidx)
            }
            Operands::Br => syntax::Operands::LabelIndex(self.read_index_use()?),
            Operands::BrTable => {
                let idxs = self.read_vec(|_, s| s.read_index_use())?;
                let last = self.read_index_use()?;
                syntax::Operands::BrTable(idxs, last)
            }
            Operands::I32 => syntax::Operands::I32(self.read_i32_leb_128().result(self)? as u32),
            Operands::I64 => syntax::Operands::I64(self.read_i64_leb_128().result(self)? as u64),
            Operands::F32 => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf).result(self)?;
                let val = f32::from_bits(u32::from_le_bytes(buf));
                syntax::Operands::F32(val)
            }
            Operands::F64 => {
                let mut buf = [0u8; 8];
                self.read_exact(&mut buf).result(self)?;
                let val = f64::from_bits(u64::from_le_bytes(buf));
                syntax::Operands::F64(val)
            }
            Operands::Memargs => syntax::Operands::Memargs(
                self.read_u32_leb_128().result(self)?,
                self.read_u32_leb_128().result(self)?,
            ),
            Operands::MemoryInit => {
                data_indices_ok
                    .true_or_else(|| self.err(BinaryParseErrorKind::DataCountMissing))?;
                self.read_zero_byte()?;
                syntax::Operands::None
            }
            Operands::MemorySize | Operands::MemoryGrow | Operands::MemoryFill => {
                self.read_zero_byte()?;
                syntax::Operands::None
            }
            Operands::MemoryCopy => {
                self.read_zero_byte()?;
                self.read_zero_byte()?;
                syntax::Operands::None
            }
            Operands::Block => {
                let bt = self.read_blocktype()?;
                let expr = self.read_expr(data_indices_ok)?;
                syntax::Operands::Block(None, bt, expr, Continuation::End)
            }
            Operands::Loop => {
                let bt = self.read_blocktype()?;
                let expr = self.read_expr(data_indices_ok)?;
                syntax::Operands::Block(None, bt, expr, Continuation::Start)
            }
            Operands::If => {
                let bt = self.read_blocktype()?;
                let th = self.read_expr_with_end(data_indices_ok)?;
                let el = if matches!(th.end, ExpressionEnd::Else) {
                    self.read_expr(data_indices_ok)?
                } else {
                    UncompiledExpr { instr: vec![] }
                };
                syntax::Operands::If(None, bt, th.expr, el)
            }
            Operands::HeapType => {
                let ht = self.read_ref_type()?;
                syntax::Operands::HeapType(ht)
            }
            Operands::CallIndirect => {
                let typeuse = self.read_type_use()?;
                let tabidx = self.read_index_use()?;
                syntax::Operands::CallIndirect(tabidx, typeuse)
            }
            Operands::SelectT => {
                let ts = self
                    .read_vec(|_, s| s.read_value_type())?
                    .iter()
                    .map(|t| FResult { valuetype: *t })
                    .collect();
                syntax::Operands::SelectT(ts)
            }
        };

        Ok(InstructionOrEnd::Instruction(Instruction {
            name: Id::default(),
            opcode,
            operands,
            location,
        }))
    }
}
