//! Methods for emitting the table of execution functions.

use {
    super::Instruction,
    crate::InstructionsForVariant,
    std::io::{Result, Write},
};

pub trait EmitExecTable: Write + std::fmt::Debug {
    /// Emit the file containing the lookup table array. It generates an array
    /// with 256 entries, and each entry in the array corresponds to one
    /// opcode.
    fn emit_exec_table(&mut self, inst_groups: &[InstructionsForVariant]) -> Result<()> {
        self.write_all(EXEC_TABLE_IMPORTS)?;
        for insts in inst_groups {
            self.write_all(exec_table_open(insts.variant.prefix()).as_bytes())?;
            for i in 0usize..=255 {
                self.write_all(exec_table_item(&insts.instructions[i]).as_bytes())?;
            }

            self.write_all(b"];\n")?;
        }

        Ok(())
    }
}

impl<W: Write + std::fmt::Debug> EmitExecTable for W {}

/// Emit one time in the lookup table. If the item is [None], the `bad` method
/// will be used, which should be implemented by the target module. Instructions
/// with en empty body emit `unimpl` as a helpful reminder to the developer.
fn exec_table_item(inst: &Option<Instruction>) -> String {
    match inst {
        None => "    bad,\n".into(),
        Some(i) => format!("    {}_exec,\n", i.typename),
    }
}

static EXEC_TABLE_IMPORTS: &[u8] = br#"use crate::instructions::code::*;
use crate::instructions::{bad, ExecFn};
"#;

fn exec_table_open(prefix: &str) -> String {
    format!(
        "#[rustfmt::skip]\npub static {}EXEC_TABLE: &[ExecFn] = &[\n",
        prefix
    )
}
