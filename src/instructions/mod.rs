pub mod opconsts;
pub mod gentypes;

use crate::runtime::exec::ExecutionContextActions;
use crate::error::Result;

pub type Expr = [u8];

#[allow(dead_code)]
enum ParseArgs {
    None,
    U32,
    U32U32,
    VU32,
    VU32U32,
    D8,
    U64,
    F32,
    F64,
    D8D8,
    U32D8
}

#[allow(dead_code)]
pub struct InstructionData {
    opcode: u8,
    name: &'static str,
    parse_args: ParseArgs
}

pub trait Instruction {
    fn data() -> InstructionData;
    fn exec<E:ExecutionContextActions>(ec: &mut E) -> Result<()>;
}
