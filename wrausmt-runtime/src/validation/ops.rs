use {
    super::{Result, Validation, ValidationError, ValidationMode},
    crate::syntax::{Instruction, Opcode, Resolved},
};

impl<'a> Validation<'a> {
    pub fn handle_instr(&mut self, instr: &Instruction<Resolved>) -> Result<()> {
        let result = match instr.opcode {
            Opcode::Normal(oc) => match oc {
                0x00 => self.unreachable(),
                _ => Err(ValidationError::UnknownOpcode),
            },
            Opcode::Extended(_) => Ok(()),
            Opcode::Simd(_) => Ok(()),
        };
        match self.context.mode {
            ValidationMode::Warn => {
                println!("Validation Failed: {result:?}");
                Ok(())
            }
            ValidationMode::Fail => result,
            ValidationMode::Panic => panic!("Validation failed: {result:?}"),
        }
    }
}
