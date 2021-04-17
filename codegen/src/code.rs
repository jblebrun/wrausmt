use super::Instruction;
use std::collections::HashMap;
use std::io::{Result, Write};

/// Emit the execution functions for all of the [Instructions][Instruction] in the provided set.
pub trait EmitCode: Write + std::fmt::Debug {
    fn emit_code_file(&mut self, insts: &HashMap<u32, Instruction>) -> Result<()> {
        self.write_all(CODE_HEADER.as_bytes())?;

        for (_, inst) in insts.iter() {
            let code = code_item(inst);
            self.write_all(code.as_bytes())?;
        }

        Ok(())
    }
}

impl <W:Write+std::fmt::Debug> EmitCode for W {}

pub static CODE_HEADER: &str = &"
use crate::runtime::exec::ExecutionContext;
use crate::runtime::exec::ExecutionContextActions;
use crate::error::Result;
";

fn code_item(inst: &Instruction) -> String {
    format!(
        "
#[allow(dead_code)]
pub fn {typename}_exec(_ec: &mut ExecutionContext) -> Result<()> {{
  {body}    Ok(())
}}
",
        typename = inst.typename,
        body = inst.body,
    )
}
