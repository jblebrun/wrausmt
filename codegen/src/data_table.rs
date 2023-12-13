//! Functions to emit the table of wrausmt::instruction::InstructionData items.

use super::Instruction;
use std::collections::HashMap;
use std::io::Result;
use std::io::Write;

pub static DATA_TABLE_HEADER: &str = "use crate::instructions::InstructionData;
use crate::instructions::Operands;
use crate::instructions::BAD_INSTRUCTION;

pub static INSTRUCTION_DATA: &[InstructionData] = &[
";

pub trait EmitDataTable: Write {
    /// Emit the code for the table of [InstructionData][wrausmt::instruction::InstructionData]
    /// items, in opcode order.
    fn emit_instruction_data_table(&mut self, insts: &HashMap<u8, Instruction>) -> Result<()> {
        self.write_all(DATA_TABLE_HEADER.as_bytes())?;

        for i in 0u8..=255 {
            self.write_all(data_table_item(insts.get(&i)).as_bytes())?;
        }

        self.write_all("];\n".as_bytes())?;

        Ok(())
    }
}

impl<W: Write> EmitDataTable for W {}

/// Emit one [InstructionData] item. If the provided entry is [None], "BAD_INSTRUCTION" is used,
/// which needs to be defined in the target module.
fn data_table_item(inst: Option<&Instruction>) -> String {
    match inst {
        Some(i) => format!(
            "    InstructionData {{
        opcode: {:#x},
        name: \"{}\",
        operands: {},
    }},\n",
            i.opcode, i.name, i.operands
        ),
        _ => "    BAD_INSTRUCTION,\n".into(),
    }
}
