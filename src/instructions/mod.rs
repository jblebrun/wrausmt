#[derive(Debug, Clone)]
pub struct Opcode {}

macro_rules! opcodes {
    ( $( $name:ident = $opcode:expr ),* ) => {
        $(
            #[allow(non_upper_case_globals)]
            #[allow(dead_code)]
            pub const $name:u8 = $opcode;
        )*
    }
}


opcodes! {
    // Control
    Nop = 0x01,
    Block = 0x02,
    Loop = 0x03,
    If = 0x04,
    Else = 0x05,
    End = 0x0B,
    Br = 0x0C,
    BrIf = 0x0D,
    BrTable = 0x0E,
    Return = 0x0F,
    Call = 0x10,
    CallIndirect = 0x11,
    
    // Parametric
    Drop = 0x1A,
    Select = 0x1B,
    SelectT = 0x1C,

    // Reference
    RefNull = 0xD0,
    RefIsNull = 0xD1,
    RefFunc = 0xD2,

    // Locals
    LocalGet = 0x20,
    LocalSet = 0x21,
    LocalTee = 0x22,
    GlobalGet = 0x23,
    GlobalSet = 0x24,

    // Table
    TableGet = 0x25,
    TableSet = 0x26,
    // Also extended, see below

    // Memory
    I32Load = 0x28,
    I64Load = 0x29,
    F32Load = 0x2A,
    F64Load = 0x2B,

    I32Load8s = 0x2C,
    I32Load8u = 0x2D,
    I32Load16s = 0x2E,
    I32Load16u = 0x2F,
    
    I64Load8s = 0x30,
    I64Load8u = 0x31,
    I64Load16s = 0x32,
    I64Load16u = 0x33,
    I64Load32s = 0x34,
    I64Load32u = 0x35,

    I32Store = 0x36,
    I64Store = 0x37,
    F32Store = 0x38,
    F64Store = 0x39,
    
    I32store8 = 0x3A,
    I32store16 = 0x3B,
    I64Store8 = 0x3C,
    I64Store16 = 0x3D,
    I64Store32= 0x3E,

    MemorySize = 0x3F,
    MemoryGrow = 0x40,
    // Also extended, see below
    
    // Numeric
    I32Const = 0x41,
    I64Const = 0x42,
    F32Const = 0x43,
    F64Const = 0x44,

    I32Eqz = 0x45,
    I32Eq  = 0x46,
    I32Ne  = 0x47,
    I32LtS = 0x48,
    I32LtU = 0x49,
    I32GtS = 0x4A,
    I32GtU = 0x4B,
    I32LeS = 0x4C,
    I32LeU = 0x4D,
    I32GeS = 0x4E,
    I32GeU = 0x4F,

    I64Eqz = 0x50,
    I64Eq  = 0x51,
    I64Ne  = 0x52,
    I64LtS = 0x53,
    I64LtU = 0x54,
    I64GtS = 0x55,
    I64GtU = 0x56,
    I64LeS = 0x57,
    I64LeU = 0x58,
    I64GeS = 0x59,
    I64GeU = 0x5A,

    F32Eq = 0x5B,
    F32Ne = 0x5C,
    F32Lt = 0x5D,
    F32Gt = 0x5E,
    F32Le = 0x5F,
    F32Ge = 0x60,
    
    F64Eq = 0x61,
    F64Ne = 0x62,
    F64Lt = 0x63,
    F64Gt = 0x64,
    F64Le = 0x65,
    F64Ge = 0x66,

    I32Clz    = 0x67,
    I32Ctz    = 0x68,
    I32Popcnt = 0x69,
    I32Add    = 0x6A,
    I32Sub    = 0x6B,
    I32Mul    = 0x6C,
    I32DivS   = 0x6D,
    I32DivU   = 0x6E,
    I32RemS   = 0x6F,
    I32RemU   = 0x70,
    I32And    = 0x71,
    I32Or     = 0x72,
    I32Xor    = 0x73,
    I32Shl    = 0x74,
    I32ShrS   = 0x75,
    I32ShrU   = 0x76,
    I32Rotl   = 0x77,
    I32Rotr   = 0x78,

    I64Clz    = 0x79,
    I64Ctz    = 0x7A,
    I64Popcnt = 0x7B,
    I64Add    = 0x7C,
    I64Sub    = 0x7D,
    I64Mul    = 0x7E,
    I64DivS   = 0x7F,
    I64DivU   = 0x80,
    I64RemS   = 0x81,
    I64RemU   = 0x82,
    I64And    = 0x83,
    I64Or     = 0x84,
    I64Xor    = 0x85,
    I64Shl    = 0x86,
    I64ShrS   = 0x87,
    I64ShrU   = 0x88,
    I64Rotl   = 0x89,
    I64Rotr   = 0x8A,

    F32Abs      = 0x8B,
    F32Neg      = 0x8C,
    F32Ceil     = 0x8D,
    F32Floor    = 0x8E,
    F32Trunc    = 0x8F,
    F32Nearest  = 0x90,
    F32Sqrt     = 0x91,
    F32Add      = 0x92,
    F32Sub      = 0x93,
    F32Mul      = 0x94,
    F32Div      = 0x95,
    F32Min      = 0x96,
    F32Max      = 0x97,
    F32Copysign = 0x98,

    F64Abs      = 0x99,
    F64Neg      = 0x9A,
    F64Ceil     = 0x9B,
    F64Floor    = 0x9C,
    F64Trunc    = 0x9D,
    F64Nearest  = 0x9E,
    F64Sqrt     = 0x9F,
    F64Add      = 0xA0,
    F64Sub      = 0xA1,
    F64Mul      = 0xA2,
    F64Div      = 0xA3,
    F64Min      = 0xA4,
    F64Max      = 0xA5,
    F64Copysign = 0xA6,

    I32WrapI64  = 0xA7,

    I32TruncF32S = 0xA8,
    I32TruncF32U = 0xA9,
    I32TruncF64S = 0xAA,
    I32TruncF64U = 0xAB,

    I64ExtendI32S = 0xAC,
    I64ExtendI32U = 0xAD,

    I64TruncF32S = 0xAE,
    I64TruncF32U = 0xAF,
    I64TruncF64S = 0xB0,
    I64TruncF64U = 0xB1,

    F32ConvertI32S = 0xB2,
    F32ConvertI32U = 0xB3,
    F32ConvertI64S = 0xB4,
    F32ConvertI64U = 0xB5,
    
    F32DemoteF64 = 0xB6,

    F64ConvertI32S = 0xB7,
    F64ConvertI32U = 0xB8,
    F64ConvertI64S = 0xB9,
    F64ConvertI64U = 0xBA,
    
    F64PromoteF32 = 0xBB,

    I32ReinterpretF32 = 0xBC,
    I64ReinterpretF64 = 0xBD,
    F32ReinterpretI32 = 0xBE,
    F64ReinterpretI64 = 0xBF,

    I32Extend8S  = 0xC0,
    I32Extend16S = 0xC1,
    I64Extend8S  = 0xC2,
    I64Extend16S = 0xC3,
    I64Extend32S = 0xC4,
    // Extended instructions under 0xFC


    // Extended Memory, Table
    Extended = 0xFC
}

// Secondary opcode byte for extended table/memory instructions.
opcodes! {
    I32TruncSatF32S = 0x00,
    I32TruncSatF32U = 0x01,
    I32TruncSatF64S = 0x02,
    I32TruncSatF64U = 0x03,
    I64TruncSatF32S = 0x04,
    I64TruncSatF32U = 0x05,
    I64TruncSatF64S = 0x06,
    I64TruncSatF64U = 0x07,

    MemoryInit = 0x08,
    DataDrop = 0x09,
    MemoryCopy = 0x0A,
    MemoryFill =0x0B,

    TableInit = 0x0C,
    ElemDrop = 0x0D,
    TableCopy = 0x0E,
    TableGrow = 0x0F,
    TableSize = 0x10,
    TableFill = 0x11
}
