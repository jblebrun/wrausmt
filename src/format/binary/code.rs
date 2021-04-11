use std::io::{Read, Write};
use super::{
    ensure_consumed::EnsureConsumed, 
    values::ReadWasmValues
};
use crate::{
    module::Function,
    types::ValueType,
    error::{Result, ResultFrom},
    instructions::*,
    err,
};

/// Read the Code section of a binary module.
/// codesec := section vec(code)
/// code := size:u32 code:func
/// func := (t*)*:vec(locals) e:expr
/// locals := n:u32 t:type
/// expr := (instr)*
pub trait ReadCode : ReadWasmValues {
    fn read_code_section(&mut self) -> Result<Box<[Function]>> {
        self.read_vec(|_, s| { s.read_func().wrap("reading func") })
    }

    fn read_vec_exprs(&mut self) -> Result<Box<[Box<Expr>]>> {
        self.read_vec(|_, s| { s.read_expr().wrap("reading expr") })
    }

    /// code := size:u32 code:func
    /// func := (t*)*:vec(locals) e:expr
    /// The size is the size in bytes of the entire section, locals + exprs
    fn read_func(&mut self) -> Result<Function> {
        let codesize = self.read_u32_leb_128().wrap("parsing func")?;
        let mut code_reader = self.take(codesize as u64);
        let function = Function {
            // The types are parsed earlier and will be set on the returned values.
            functype: 0,
            locals: code_reader.read_locals().wrap("parsing locals")?,
            body: code_reader.read_expr().wrap("parsing code")?
        };
        code_reader.ensure_consumed()?;
        Ok(function)
    }

    /// Read the locals description for the function.
    /// locals := n:u32 t:type
    fn read_locals(&mut self) -> Result<Box<[ValueType]>> {
        let items = self.read_u32_leb_128().wrap("parsing item count")?;
        let mut result: Vec<ValueType> = vec![];

        for _ in 0..items {
            let reps = self.read_u32_leb_128().wrap("parsing type rep")?;
            let val = self.read_value_type().wrap("parsing value type")?;
            for _ in 0..reps {
                result.push(val);
            }
        }
        Ok(result.into_boxed_slice())
    }

    /// Read the instructions from one function in the code section.
    /// The code is stored in the module as raw bytes, with LEB128 numbers
    /// converted to little-endian format.
    /// expr := (instr)*
    fn read_expr(&mut self) -> Result<Box<[u8]>> {
        let mut result: Vec<u8> = vec![];
        while self.read_inst(&mut result).wrap("read inst byte")? == 1 {}
        Ok(result.into_boxed_slice())
    }

    /// Returns 0 if EOF was reached while parsing an opcode.
    /// Returns 1 if a full instruction was parsed.
    /// Returns Err result otherwise.
    fn read_inst<W : Write>(&mut self, out: &mut W) -> Result<usize> {
        let mut opcode_buf = [0u8; 1];
        let cnt = self.read(&mut opcode_buf).wrap("parsing opcode")?;
        if cnt == 0 {
            return Ok(0)
        }
        let opcode = opcode_buf[0];
        
        // Assume success, write out the opcode. Validation occurs later.
        out.write(&opcode_buf).wrap("writing opcode")?;

        // Handle any additional behavior
        #[allow(non_upper_case_globals)]
        match opcode {
            Block => self.read_1_arg(out)?,
            BrIf => self.read_1_arg(out)?,
            Return => (),
            Call => self.read_1_arg(out)?,
            CallIndirect => self.read_2_args(out)?,
            End => (),
            LocalGet => self.read_1_arg(out)?,
            LocalSet => self.read_1_arg(out)?,
            GlobalGet => self.read_1_arg(out)?,
            I32Load => self.read_2_args(out)?,
            I32Store => self.read_2_args(out)?,
            I32Const => self.read_1_arg(out)?,
            F32Const => self.read_1_arg(out)?,
            I32Add => (),
            I32Sub => (),
            _ => return err!("unknown opcode 0x{:x?}", opcode)
        }
        Ok(1)
    }

    /// Clarity method: use to read a single LEB128 argument for an instruction.
    fn read_1_arg<W : Write>(&mut self, out: &mut W) -> Result<()> {
        self.emit_read_u32_leb_128(out).wrap("parsing arg 1/1")
    }

    /// Clarity method: use to read a two successive LEB128 arguments for an instruction.
    fn read_2_args<W : Write>(&mut self, out: &mut W) -> Result<()> {
        self.emit_read_u32_leb_128(out).wrap("parsing arg 1/2")?;
        self.emit_read_u32_leb_128(out).wrap("arsing arg 2/2")
    }

    /// Read one LEB128 value and emit it to the provided writer.
    fn emit_read_u32_leb_128<W : Write>(&mut self, out: &mut W) -> Result<()> {
        out.write(
            &self.read_u32_leb_128().wrap("reading leb 128")?.to_le_bytes()
        ).wrap("writing leb 128")?;
        Ok(())
    }
}

impl <I:ReadWasmValues> ReadCode for I {}

