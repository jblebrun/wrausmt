//! Implementations of the instructions that are executed in the body of a
//! WebAssembly function. Most of the code for instruction execution is
//! generated. See the [codegen] crate for more details on the generation
//! process.

use {
    self::generated::{
        data_table::{EXTENDED_INSTRUCTION_DATA, INSTRUCTION_DATA, SIMD_INSTRUCTION_DATA},
        exec_table::{EXEC_TABLE, EXTENDED_EXEC_TABLE, SIMD_EXEC_TABLE},
    },
    crate::{
        impl_bug,
        runtime::{error::Result, exec::ExecutionContext},
        syntax::{Id, Opcode},
    },
};

mod generated;

/// Function bodies, initialization values for globals, and offsets of element
/// or data segments are given as expressions, which are sequences of
/// instructions terminated by an end marker.
///
/// [Spec](https://webassembly.github.io/spec/core/syntax/instructions.html#expressions)
pub type Expr = [u8];

/// Information about one assembly instruction, used during parsing.
///
/// The `opcode` field contains the byte used to represent the instruction in
/// the WebAssembly format. The core instructions are one byte, but some are two
/// bytes, with a prefix of 0xFC or 0xFD. The Opcode enum will contain an enum
/// variant to select the space, and each variant holds the byte for opcode
/// selection in that space.
///
/// Tne `name` field contains the string name of the operation, as it appears in
/// WebAssembly text format.
///
/// The `operands` field is in item from the [Operands] enum, describing the
/// number of immediate operands to expect for this instruction, also used to
/// guide parsing.
#[derive(PartialEq, Debug)]
pub struct InstructionData {
    pub opcode:   Opcode,
    pub name:     &'static str,
    pub operands: Operands,
}

/// An enum representing the different combinations of immediate operands that a
/// WebAssembly instruction can have.
#[derive(PartialEq, Debug)]
pub enum Operands {
    None,
    Block,
    Loop,
    If,
    Br,
    BrTable,
    CallIndirect,
    Select,
    SelectT,
    HeapType,
    FuncIndex,
    LocalIndex,
    GlobalIndex,
    TableIndex,
    MemoryIndex,
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

/// A method for executing a function in the given provided [ExecutionContext].
/// Each method directly manages its own operand acquisition, pushing, and
/// popping via the operators available in the provided [ExecutionContext];
/// there are no generalized conveniences for generating different
/// function types for the different groups of instructions.
pub type ExecFn = fn(ec: &mut ExecutionContext) -> Result<()>;

/// This function appears in the lookup table for opcodes that don't have a
/// corresponding operation in the specification.
pub fn bad(_ec: &mut ExecutionContext) -> Result<()> {
    Err(impl_bug!("unknown opcode"))?
}

/// This function is used to mark not-yet-implemented instructions in the table.
pub fn unimpl(_ec: &mut ExecutionContext) -> Result<()> {
    Err(impl_bug!("not yet implemented"))?
}

pub const BAD_INSTRUCTION: InstructionData = InstructionData {
    opcode:   Opcode::Normal(0),
    name:     "bad",
    operands: Operands::None,
};

pub fn exec_method(opcode: Opcode, ec: &mut ExecutionContext) -> Result<()> {
    let exec_fn = match opcode {
        Opcode::Extended(o) => EXTENDED_EXEC_TABLE.get(o as usize),
        Opcode::Simd(o) => SIMD_EXEC_TABLE.get(o as usize),
        Opcode::Normal(o) => EXEC_TABLE.get(o as usize),
    };
    match exec_fn {
        Some(ef) => ef(ec),
        None => Err(impl_bug!("Exec table short"))?,
    }
}

pub fn instruction_data(opcode: &Opcode) -> &'static InstructionData {
    match *opcode {
        Opcode::Normal(o) => &INSTRUCTION_DATA[o as usize],
        Opcode::Extended(o) => &EXTENDED_INSTRUCTION_DATA[o as usize],
        Opcode::Simd(o) => &SIMD_INSTRUCTION_DATA[o as usize],
    }
}

// TODO - would it be significantly more performant to build a hash map here?
// Or maybe just a two-tiered lookup.
pub fn instruction_by_name(name: &Id) -> Option<&'static InstructionData> {
    INSTRUCTION_DATA
        .iter()
        .chain(EXTENDED_INSTRUCTION_DATA.iter())
        .chain(SIMD_INSTRUCTION_DATA.iter())
        .find(|&item| item.name == name.as_str())
}

pub mod op_consts {
    pub const EXTENDED_PREFIX: u8 = 0xFC;
    pub const SIMD_PREFIX: u8 = 0xFD;
}
