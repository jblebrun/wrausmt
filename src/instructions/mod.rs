pub mod opconsts;
pub mod gentypes;
pub mod exec_table;

use crate::runtime::exec::ExecutionContextActions;
use crate::error::Result;

use self::gentypes::LocalGet;

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
    fn exec(ec: &mut ExecutionContext) -> Result<()>;
}

use crate::runtime::exec::ExecutionContext;
use crate::err;
pub type ExecFn = fn(ec: &mut ExecutionContext) -> Result<()>;

pub fn bad(ec: &mut ExecutionContext) -> Result<()> {
    err!("unknown oopcode")
}

pub fn unimpl(ec: &mut ExecutionContext) -> Result<()> {
    err!("not yet implemented")
}

static EXEC_TABLE: &[ExecFn] = &[
    bad,
    gentypes::Nop::exec,

];
