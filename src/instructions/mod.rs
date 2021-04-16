mod generated;

use crate::error::{Result, ResultFrom};
use generated::exec_table::EXEC_TABLE;
use generated::data_table::INSTRUCTION_DATA;
use crate::runtime::exec::ExecutionContext;
use crate::err;

pub type Expr = [u8];

#[derive(PartialEq, Debug)]
pub enum Operands {
    None,
    U32,
    U32U32,
    Vu32,
    Vu32U32,
    D8,
    U64,
    F32,
    F64,
    D8D8,
    U32D8
}

#[derive(PartialEq, Debug)]
pub struct InstructionData {
    pub opcode: u8,
    pub name: &'static str,
    pub operands: Operands 
}

pub type ExecFn = fn(ec: &mut ExecutionContext) -> Result<()>;

pub fn bad(_ec: &mut ExecutionContext) -> Result<()> {
    err!("unknown opcode")
}

pub fn unimpl(_ec: &mut ExecutionContext) -> Result<()> {
    err!("not yet implemented")
}

pub const BAD_INSTRUCTION: InstructionData = InstructionData{
    opcode: 0, 
    name: "bad", 
    operands: Operands::None, 
};

pub fn exec_method(opcode: u8, ec: &mut ExecutionContext) -> Result<()> {
    match EXEC_TABLE.get(opcode as usize) {
        Some(ef) => ef(ec).wrap(&format!("while executing 0x{:x}", opcode)),
        None => err!("unhandled opcode {}", opcode)
    }
}

pub fn instruction_data<'l>(opcode: u8) -> Result<&'l InstructionData> {
    match INSTRUCTION_DATA.get(opcode as usize) {
        Some(id) if id == &&BAD_INSTRUCTION => err!("invalid instruction {}", opcode), 
        Some(id) => Ok(id), 
        None => err!("unhandled opcode {}", opcode)
    }
}
