use super::{ensure_consumed::EnsureConsumed, error::BinaryParseError, values::ReadWasmValues};
use crate::{
    instructions::{instruction_data, Operands, BAD_INSTRUCTION},
    syntax::{self, Continuation, Expr, FuncField, Instruction, Local, Resolved, TypeUse},
};
use std::{
    collections::HashMap,
    io::{Read, Write},
};

use super::error::{Result, WithContext};

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
    end: ExpressionEnd,
}

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
            id: None,
            exports: vec![],
            // The types are parsed earlier and will be set on the returned values.
            typeuse: TypeUse::default(),
            locals: code_reader.read_locals().ctx("parsing locals")?,
            body: code_reader.read_expr().ctx("parsing code")?,
            localindices: HashMap::default(),
        };
        code_reader.ensure_consumed()?;
        Ok(function)
    }

    /// Read the locals description for the function.
    /// locals := n:u32 t:type
    fn read_locals(&mut self) -> Result<Vec<Local>> {
        let items = self.read_u32_leb_128().ctx("parsing item count")?;
        let mut result: Vec<Local> = vec![];

        for _ in 0..items {
            let reps = self.read_u32_leb_128().ctx("parsing type rep")?;
            let val = self.read_value_type().ctx("parsing value type")?;
            for _ in 0..reps {
                result.push(Local {
                    id: None,
                    valtype: val,
                });
            }
        }
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

    /// Returns -1 if EOF or end instruction was reached while parsing an opcode.
    /// Returns 1 if a new block was started
    /// Returns 0 if a normal instruction was parsed.
    /// Returns Err result otherwise.
    fn read_inst(&mut self) -> Result<InstructionOrEnd> {
        let mut opcode_buf = [0u8; 1];
        self.read_exact(&mut opcode_buf).ctx("parsing opcode")?;

        // 0xFC instructions are shifted into the normal opcode
        // table starting at 0xE0.
        let opcode = if opcode_buf[0] == 0xFC {
            let second_opcode = self.read_u32_leb_128().ctx("parsing secondary opcode")?;
            second_opcode as u8 + 0xE0
        } else {
            opcode_buf[0]
        };

        let instruction_data = instruction_data(opcode);

        if instruction_data == &BAD_INSTRUCTION {
            return Err(BinaryParseError::InvalidOpcode(opcode));
        }

        // End of expression.
        if opcode == 0x0B {
            return Ok(InstructionOrEnd::End(ExpressionEnd::End));
        }
        if opcode == 0x05 {
            return Ok(InstructionOrEnd::End(ExpressionEnd::Else));
        }

        // Handle any additional behavior
        let operands = match instruction_data.operands {
            Operands::None => syntax::Operands::None,
            Operands::FuncIndex => syntax::Operands::FuncIndex(self.read_index_use()?),
            Operands::LocalIndex => syntax::Operands::FuncIndex(self.read_index_use()?),
            Operands::GlobalIndex => syntax::Operands::FuncIndex(self.read_index_use()?),
            Operands::TableIndex => syntax::Operands::FuncIndex(self.read_index_use()?),
            Operands::MemIndex => syntax::Operands::FuncIndex(self.read_index_use()?),
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
                    "unsupported operands {:x?} for {:x}",
                    instruction_data.operands, opcode
                )
            }
        };

        Ok(InstructionOrEnd::Instruction(Instruction {
            name: "".to_owned(),
            opcode,
            operands,
        }))
    }

    /// Clarity method: use to read a single LEB128 argument for an instruction.
    fn read_u32_arg<W: Write>(&mut self, out: &mut W) -> Result<()> {
        self.emit_read_u32_leb_128(out).ctx("parsing arg 1/1")
    }

    /// Read one LEB128 value and emit it to the provided writer.
    fn emit_read_u32_leb_128<W: Write>(&mut self, out: &mut W) -> Result<()> {
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
