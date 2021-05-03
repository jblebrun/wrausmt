use super::Instruction;
use std::collections::HashMap;
use std::io::{Result, Write};

/// Emit the execution functions for all of the [Instructions][Instruction] in the provided set.
pub trait EmitCode: Write + std::fmt::Debug {
    fn emit_code_file(&mut self, insts: &HashMap<u8, Instruction>) -> Result<()> {
        self.write_all(CODE_HEADER.as_bytes())?;

        for (_, inst) in insts.iter() {
            if !inst.body.is_empty() {
                let code = code_item(inst);
                self.write_all(code.as_bytes())?;
            }
        }

        Ok(())
    }
}

impl<W: Write + std::fmt::Debug> EmitCode for W {}

pub static CODE_HEADER: &str = &"use crate::error::Result;
use crate::runtime::exec::ExecutionContextActions;
use crate::runtime::Runtime;
";

fn code_item(inst: &Instruction) -> String {
    format!(
        "
#[allow(dead_code)]
pub fn {typename}_exec(_ec: &mut Runtime) -> Result<()> {{
  {body}    
}}
",
        typename = inst.typename,
        body = inst.body,
    )
}
