//! Functions to emit the table of wrausmt_runtime::instruction::InstructionData
//! items.

use {
    super::Instruction,
    crate::{InstructionsForVariant, Variant},
    std::io::{Result, Write},
};

pub static IMPORTS: &[u8] = br#"use crate::instructions::InstructionData;
use crate::instructions::{Opcode, Operands, BAD_INSTRUCTION};
"#;

fn data_table_header(prefix: &str) -> String {
    format!(
        "pub static {}INSTRUCTION_DATA: &[InstructionData] = &[\n",
        prefix
    )
}

pub trait EmitDataTable: Write {
    /// Emit the code for the table of
    /// [InstructionData][wrausmt_runtime::instruction::InstructionData]
    /// items, in opcode order.
    fn emit_instruction_data_table(
        &mut self,
        inst_groups: &[InstructionsForVariant],
    ) -> Result<()> {
        self.write_all(IMPORTS)?;

        for insts in inst_groups.iter() {
            self.write_all(data_table_header(insts.variant.prefix()).as_bytes())?;

            for i in 0_usize..=255 {
                self.write_all(data_table_item(&insts.instructions[i], &insts.variant).as_bytes())?;
            }

            self.write_all(b"];\n")?;
        }

        Ok(())
    }
}

impl<W: Write> EmitDataTable for W {}

/// Emit one [InstructionData] item. If the provided entry is [None],
/// "BAD_INSTRUCTION" is used, which needs to be defined in the target module.
fn data_table_item(inst: &Option<Instruction>, variant: &Variant) -> String {
    match inst {
        Some(i) => format!(
            "    InstructionData {{
        opcode:   {}({:#x}),
        name:     \"{}\",
        operands: {},
    }},\n",
            variant.opcode_variant(),
            i.opcode,
            i.name,
            i.operands
        ),
        _ => "    BAD_INSTRUCTION,\n".into(),
    }
}
