//! Methods for emitting the table of execution functions.
use {
    super::Instruction,
    std::{
        collections::HashMap,
        io::{Result, Write},
    },
};

pub trait EmitExecTable: Write + std::fmt::Debug {
    /// Emit the file containing the lookup table array. It generates an array
    /// with 256 entries, and each entry in the array corresponds to one
    /// opcode.
    fn emit_exec_table(&mut self, insts: &HashMap<u8, Instruction>) -> Result<()> {
        self.write_all(EXEC_TABLE_HEADER.as_bytes())?;

        for i in 0u8..=255 {
            self.write_all(exec_table_item(insts.get(&i)).as_bytes())?;
        }

        self.write_all("];\n".as_bytes())?;

        Ok(())
    }
}

impl<W: Write + std::fmt::Debug> EmitExecTable for W {}

/// Emit one time in the lookup table. If the item is [None], the `bad` method
/// will be used, which should be implemented by the target module. Instructions
/// with en empty body emit `unimpl` as a helpful reminder to the developer.
fn exec_table_item(inst: Option<&Instruction>) -> String {
    match inst {
        None => "    bad,\n".into(),
        Some(i) if i.body.is_empty() => "    unimpl,\n".into(),
        Some(i) => format!("    instructions::{}_exec,\n", i.typename),
    }
}

pub static EXEC_TABLE_HEADER: &str = "use super::instructions;
use crate::instructions::{bad, unimpl, ExecFn};

pub static EXEC_TABLE: &[ExecFn] = &[
";
