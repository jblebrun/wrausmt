use super::{ensure_consumed::EnsureConsumed, values::ReadWasmValues};
use crate::{
    err,
    error::{Result, ResultFrom},
    instructions::*,
    module::Function,
    types::ValueType,
};
use std::io::{Read, Write};
use crate::instructions::instruction_data;

/// Read the Code section of a binary module.
/// codesec := section vec(code)
/// code := size:u32 code:func
/// func := (t*)*:vec(locals) e:expr
/// locals := n:u32 t:type
/// expr := (instr)*
pub trait ReadCode: ReadWasmValues {
    fn read_code_section(&mut self) -> Result<Box<[Function]>> {
        self.read_vec(|_, s| s.read_func().wrap("reading func"))
    }

    fn read_vec_exprs(&mut self) -> Result<Box<[Box<Expr>]>> {
        self.read_vec(|_, s| s.read_expr().wrap("reading expr"))
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
            body: code_reader.read_expr().wrap("parsing code")?,
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
    /// The code is stored in the module as raw bytes, mostly following the 
    /// same structure that it has in the binary module ,but with LEB128 numbers
    /// converted to little-endian format.
    /// expr := (instr)* 0x0B
    fn read_expr(&mut self) -> Result<Box<[u8]>> {
        let mut result: Vec<u8> = vec![];
        let mut depth = 1;
        while depth > 0 {
            depth += self.read_inst(&mut result).wrap("read inst byte")?
        }
        Ok(result.into_boxed_slice())
    }

    /// Returns -1 if EOF or end instruction was reached while parsing an opcode.
    /// Returns 1 if a new block was started
    /// Returns 0 if a normal instruction was parsed.
    /// Returns Err result otherwise.
    fn read_inst<W: Write>(&mut self, out: &mut W) -> Result<i8> {
        let mut opcode_buf = [0u8; 1];
        self.read_exact(&mut opcode_buf).wrap("parsing opcode")?;

        let opcode = opcode_buf[0];

        // Assume success, write out the opcode. Validation occurs later.
        out.write(&opcode_buf).wrap("writing opcode")?;

        let instruction_data = instruction_data(opcode)?;

        // Ending block, decrease depth
        if opcode == 0x0B {
            return Ok(-1)
        }

        // Handle any additional behavior
        #[allow(non_upper_case_globals)]
        match instruction_data.operands {
            Operands::None => (),
            Operands::U32 => self.read_1_arg(out)?,
            Operands::U32U32 => self.read_2_args(out)?,
            _ => return err!("unknown opcode 0x{:x?}", opcode),
        };

        if matches!(opcode, 0x02 | 0x03 | 0x04) {
            Ok(1)
        } else {
            Ok(0)
        }
    }

    /// Clarity method: use to read a single LEB128 argument for an instruction.
    fn read_1_arg<W: Write>(&mut self, out: &mut W) -> Result<()> {
        self.emit_read_u32_leb_128(out).wrap("parsing arg 1/1")
    }

    /// Clarity method: use to read a two successive LEB128 arguments for an instruction.
    fn read_2_args<W: Write>(&mut self, out: &mut W) -> Result<()> {
        self.emit_read_u32_leb_128(out).wrap("parsing arg 1/2")?;
        self.emit_read_u32_leb_128(out).wrap("arsing arg 2/2")
    }

    /// Read one LEB128 value and emit it to the provided writer.
    fn emit_read_u32_leb_128<W: Write>(&mut self, out: &mut W) -> Result<()> {
        out.write(
            &self
                .read_u32_leb_128()
                .wrap("reading leb 128")?
                .to_le_bytes(),
        )
        .wrap("writing leb 128")?;
        Ok(())
    }
}

impl<I: ReadWasmValues> ReadCode for I {}


#[cfg(test)]
mod test {
    use super::ReadCode;
    use crate::error::Result;

    #[test]
    fn read_expr() -> Result<()> {
        let data: &[u8] = &[0x6au8, 0x68, 0x6a, 0x68, 0x0B, 0xE0, 0xE1, 0xE2];
        let mut reader = data;
        let expr = reader.read_expr()?;
        assert_eq!(*expr, data[0..5]);
        Ok(())
    }

    #[test]
    fn read_expr_nested() -> Result<()> {
        let data: &[u8] = &[0x6au8, 0x02, 0x40, 0x68, 0x6a, 0x68, 0x0B, 0x0B, 0xE0, 0xE1, 0xE2];
        let expect: &[u8] = &[0x6au8, 0x02, 0x40, 0x00, 0x00, 0x00, 0x68, 0x6a, 0x68, 0x0B, 0x0B];
        let mut reader = data;
        let expr = reader.read_expr()?;
        assert_eq!(*expr, *expect);
        Ok(())
    }

    #[test]
    fn read_expr_early_eof() -> Result<()> {
        let data: &[u8] = &[0x6au8, 0x02, 0x40, 0x68, 0x6a, 0x68, 0x0B, 0x97, 0x98, 0x99];
        let mut reader = data;
        match reader.read_expr() {
            Ok(e) => panic!("expected error, read back {:?}", e),
            Err(e) => assert!(format!("{:?}", e).contains("read inst byte"))
        }
        Ok(())
    }

}
