use {
    super::Instruction,
    crate::{InstructionsForVariant, Variant},
    std::io::{Result, Write},
};

/// Emit the execution functions for all of the [Instructions][Instruction] in
/// the provided set.
pub trait EmitCode: Write + std::fmt::Debug {
    fn emit_code_file(&mut self, inst_groups: &[InstructionsForVariant]) -> Result<()> {
        self.write_all(CODE_HEADER)?;

        for insts in inst_groups {
            for inst in insts.instructions.iter().flatten() {
                if !inst.body.is_empty() {
                    let code = code_item(inst);
                    self.write_all(code.as_bytes())?;
                }
            }
        }

        self.write_all(b"#[allow(dead_code)]\n")?;
        self.write_all(b"pub mod opcodes {\n")?;
        self.write_all(b"    use crate::syntax::Opcode;\n")?;
        for insts in inst_groups {
            for inst in insts.instructions.iter().flatten() {
                let name = inst.typename.to_ascii_uppercase();
                let opcode = inst.opcode;
                let variant = match insts.variant {
                    Variant::Normal => "Normal",
                    Variant::Extended => "Extended",
                    Variant::Simd => "Simd",
                };
                self.write_all(
                    format!("    pub const {name}: Opcode = Opcode::{variant}(0x{opcode:0x});\n")
                        .as_bytes(),
                )?;
            }
        }
        self.write_all(b"}\n")?;

        Ok(())
    }
}

impl<W: Write + std::fmt::Debug> EmitCode for W {}

pub static CODE_HEADER: &[u8] = br#"use crate::runtime::error::Result;
use crate::runtime::{
    error::TrapKind,
    exec::{ExecutionContext, ExecutionContextActions, LabelType},
    instance::addr::Address,
    values::Ref,
};
"#;

fn code_item(inst: &Instruction) -> String {
    format!(
        "
#[allow(dead_code)]
pub fn {typename}_exec(_ec: &mut ExecutionContext) -> Result<()> {{
{body}}}
",
        typename = inst.typename,
        body = inst.body,
    )
}
