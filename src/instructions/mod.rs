//! Implementations of the instructions that are executed in the body of a WebAssembly function.
//! Most of the code for instruction execution is generated. See the [codegen](codegen) crate for
//! more details on the generation process.
mod generated;

use crate::error::{Result, ResultFrom};
use generated::exec_table::EXEC_TABLE;
use generated::data_table::INSTRUCTION_DATA;
use crate::runtime::exec::ExecutionContext;
use crate::err;

/// Function bodies, initialization values for globals, and offsets of element or data segments are
/// given as expressions, which are sequences of instructions terminated by an end marker.
/// [Spec](https://webassembly.github.io/spec/core/syntax/instructions.html#expressions)
pub type Expr = [u8];

/// Information about one assembly instruction, used during parsing. 
///
/// The `opcode` field contains the byte used to represent the instruction in the WebAssembly
/// format. In the case of extended instructions in the 0xFC family (which are stored in separate
/// tables), this will indicate the extended opcode.
///
/// Tne `name` field contains the string name of the operation, as it appears in WebAssembly text
/// format.
///
/// The `operands` field is in item from the [Operands] enum, describing the number of immediate
/// operands to expect for this instruction, also used to guide parsing.
#[derive(PartialEq, Debug)]
pub struct InstructionData {
    pub opcode: u8,
    pub name: &'static str,
    pub operands: Operands 
}

/// An enum representing the different combinations of immediate operands that a WebAssembly 
/// instruction can have. There are few enough unique combinations that each one is represented by
/// its own entry in the enum set (rather than supporting generic combinations of any of the
/// types). The names of the enum values are from the set (U32, U64, F32, F64, Vu32, D8). Vu32
/// stands for a Vector of U32 values. D8 indicates a byte which can just be dropped.
#[derive(PartialEq, Debug)]
pub enum Operands {
    None,
    Block,
    Br,
    BrTable,
    CallIndirect,
    SelectT,
    HeapType,
    FuncIndex,
    LocalIndex,
    GlobalIndex,
    TableIndex,
    MemIndex,
    Memargs,
    I32,
    I64, 
    F32, 
    F64,
    MemoryInit,
    MemorySize,
    MemoryGrow,
    DataIndex,
    MemoryCopy,
    MemoryFill,
    TableInit,
    ElemIndex,
    TableCopy,
}

/// A method for executing a function in the given provided [ExecutionContext]. Each method directly
/// manages its own operand acquisition, pushing, and popping via the operators available in the
/// provided [ExecutionContext]; there are no generalized conveniences for generating different 
/// function types for the different groups of instructions.
pub type ExecFn = fn(ec: &mut ExecutionContext) -> Result<()>;

/// This function appears in the lookup table for opcodes that don't have a
/// corresponding operation in the specification.
pub fn bad(_ec: &mut ExecutionContext) -> Result<()> {
    err!("unknown opcode")
}

/// This function is used to mark not-yet-implemented instructions in the table.
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

// TODO - would it be significantly more performant to build a hash map here?
// Or maybe just a two-tiered lookup.
pub fn instruction_by_name(name: &str) -> Option<&'static InstructionData> {
    for item in INSTRUCTION_DATA {
        if item.name == name {
            return Some(&item);
        }
    }
    None
}

