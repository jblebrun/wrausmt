pub mod opconsts;

use crate::types::{NumType, ValueType};
use crate::runtime::exec::ExecutionContextActions;
use crate::error::Result;

pub type Expr = [u8];

#[allow(dead_code)]
enum ParseArgs {
    None,
    // A u32 with special interpretation
    BlockType,
    // A vector of u32 + one u32
    BrTable,
    Single(ValueType),
    Double(ValueType),
    // For the select vector type
    Vector,
    Single0,
    DiscardByte,
    Discard2Byte,
}

#[allow(dead_code)]
struct InstructionData {
    opcode: u8,
    name: &'static str,
    parse_args: ParseArgs
}

trait Instruction {
    fn data() -> InstructionData;
    fn exec<E:ExecutionContextActions>(ec: &mut E) -> Result<()>;
}

struct LocalGet { }

impl Instruction for LocalGet {
    fn data() -> InstructionData {
        InstructionData {
            opcode: 0x20,
            name: "local.get",
            parse_args: ParseArgs::Single(ValueType::Num(NumType::I32))
        }
    }

    fn exec<E:ExecutionContextActions>(ec: &mut E) -> Result<()> {
        let idx = ec.op_u32()?;
        let val = ec.get_local(idx)?;
        ec.push_value(val)
    }
}
