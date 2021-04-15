
#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const Nop:u8 = 0x01;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const Block:u8 = 0x02;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const Loop:u8 = 0x03;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const If:u8 = 0x04;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const Else:u8 = 0x05;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const End:u8 = 0x0B;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const Br:u8 = 0x0C;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const BrIf:u8 = 0x0D;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const BrTable:u8 = 0x0E;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const Return:u8 = 0x0F;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const Call:u8 = 0x10;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const CallIndirect:u8 = 0x11;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const Drop:u8 = 0x1A;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const Select:u8 = 0x1B;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const Selectt:u8 = 0x1C;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const RefNull:u8 = 0xD0;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const RefIsNull:u8 = 0xD1;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const RefFunc:u8 = 0xD2;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const LocalGet:u8 = 0x20;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const LocalSet:u8 = 0x21;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const LocalTee:u8 = 0x22;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const GlobalGet:u8 = 0x23;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const GlobalSet:u8 = 0x24;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const TableGet:u8 = 0x25;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const TableSet:u8 = 0x26;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Load:u8 = 0x28;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Load:u8 = 0x29;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Load:u8 = 0x2a;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Load:u8 = 0x2b;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Load8S:u8 = 0x2c;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Load8U:u8 = 0x2d;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Load16S:u8 = 0x2e;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Load16U:u8 = 0x2f;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Load8S:u8 = 0x30;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Load8U:u8 = 0x31;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Load16S:u8 = 0x32;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Load16U:u8 = 0x33;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Load32S:u8 = 0x34;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Load32U:u8 = 0x35;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Store:u8 = 0x36;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Store:u8 = 0x37;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Store:u8 = 0x38;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Store:u8 = 0x39;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Store8:u8 = 0x3a;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Store16:u8 = 0x3b;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Store8:u8 = 0x3c;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Store16:u8 = 0x3D;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Store32:u8 = 0x3E;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const MemorySize:u8 = 0x3f;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const MemoryGrow:u8 = 0x40;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32const:u8 = 0x41;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64const:u8 = 0x42;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32const:u8 = 0x43;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Const:u8 = 0x44;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Eqz:u8 = 0x45;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Eq:u8 = 0x46;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Ne:u8 = 0x47;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32LtS:u8 = 0x48;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32LtU:u8 = 0x49;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32GtS:u8 = 0x4a;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32GtU:u8 = 0x4b;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32LeS:u8 = 0x4c;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32LeU:u8 = 0x4d;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32GeS:u8 = 0x4e;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32GeU:u8 = 0x4f;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Eqz:u8 = 0x50;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Eq:u8 = 0x51;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Ne:u8 = 0x52;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64LtS:u8 = 0x53;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64LtU:u8 = 0x54;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64GtS:u8 = 0x55;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64GtU:u8 = 0x56;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64LeS:u8 = 0x57;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64LeU:u8 = 0x58;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64GeS:u8 = 0x59;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64GeU:u8 = 0x5a;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Eq:u8 = 0x5b;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Ne:u8 = 0x5c;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Lt:u8 = 0x5d;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Gt:u8 = 0x5e;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Le:u8 = 0x5f;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Ge:u8 = 0x60;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Eq:u8 = 0x61;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Ne:u8 = 0x62;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Lt:u8 = 0x63;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Gt:u8 = 0x64;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Le:u8 = 0x65;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64ge:u8 = 0x66;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Clz:u8 = 0x67;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Ctz:u8 = 0x68;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Popcnt:u8 = 0x69;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Add:u8 = 0x6a;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Sub:u8 = 0x6b;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Mul:u8 = 0x6c;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32DivS:u8 = 0x6d;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32DivU:u8 = 0x6e;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32RemS:u8 = 0x6f;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32RemU:u8 = 0x70;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32And:u8 = 0x71;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Or:u8 = 0x72;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Xor:u8 = 0x73;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Shl:u8 = 0x74;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32ShrS:u8 = 0x75;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32ShrU:u8 = 0x76;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Rotl:u8 = 0x77;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Rotr:u8 = 0x78;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Clz:u8 = 0x79;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Ctz:u8 = 0x7a;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Popcnt:u8 = 0x7b;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Add:u8 = 0x7c;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Sub:u8 = 0x7d;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Mul:u8 = 0x7e;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Divs:u8 = 0x7f;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Divu:u8 = 0x80;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Rems:u8 = 0x81;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Remu:u8 = 0x82;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64And:u8 = 0x83;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Or:u8 = 0x84;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Xor:u8 = 0x85;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Shl:u8 = 0x86;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Shrs:u8 = 0x87;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Shru:u8 = 0x88;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Rotl:u8 = 0x89;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Rotr:u8 = 0x8a;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Abs:u8 = 0x8b;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Neg:u8 = 0x8c;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Ceil:u8 = 0x8d;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Floor:u8 = 0x8e;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Trunc:u8 = 0x8f;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Nearest:u8 = 0x90;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Sqrt:u8 = 0x91;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Add:u8 = 0x92;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Sub:u8 = 0x93;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Mul:u8 = 0x94;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Div:u8 = 0x95;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Min:u8 = 0x96;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Max:u8 = 0x97;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32Copysign:u8 = 0x98;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Abs:u8 = 0x99;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Neg:u8 = 0x9a;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Ceil:u8 = 0x9b;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Floor:u8 = 0x9c;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Trunc:u8 = 0x9d;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Nearest:u8 = 0x9e;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Sqrt:u8 = 0x9f;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Add:u8 = 0xa0;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Sub:u8 = 0xa1;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Mul:u8 = 0xa2;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Div:u8 = 0xa3;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Min:u8 = 0xa4;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Max:u8 = 0xa5;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Copysign:u8 = 0xa6;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32WrapI64:u8 = 0xa7;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32TruncF32S:u8 = 0xa8;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32TruncF32U:u8 = 0xa9;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32TruncF64S:u8 = 0xaa;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32TruncF64U:u8 = 0xab;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64ExtendI32S:u8 = 0xac;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64ExtendI32U:u8 = 0xad;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64TruncF32S:u8 = 0xae;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64TruncF32U:u8 = 0xaf;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64TruncF64S:u8 = 0xb0;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64TruncF64U:u8 = 0xb1;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32ConvertI32S:u8 = 0xb2;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32ConvertI32U:u8 = 0xb3;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32ConvertI64S:u8 = 0xb4;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32ConvertI64U:u8 = 0xb5;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32DemoteF64:u8 = 0xb6;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64ConvertI32S:u8 = 0xb7;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64ConvertI32U:u8 = 0xb8;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64ConvertI64S:u8 = 0xb9;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64ConvertI64U:u8 = 0xba;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64Promotef32:u8 = 0xbb;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Reinterpretf32:u8 = 0xbc;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Reinterpretf64:u8 = 0xbd;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F32ReinterpretI32:u8 = 0xbe;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const F64ReinterpretI64:u8 = 0xbf;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Extend8S:u8 = 0xc0;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32Extend16S:u8 = 0xc1;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Extend8S:u8 = 0xc2;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Extend16S:u8 = 0xc3;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64Extend32S:u8 = 0xc4;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const Extended:u8 = 0xfc;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32TruncSatF32S:u8 = 0x00;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32TruncSatF32U:u8 = 0x01;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32TruncSatF64S:u8 = 0x02;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I32TruncSatF64U:u8 = 0x03;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64TruncSatF32S:u8 = 0x04;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64TruncSatF32U:u8 = 0x05;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64TruncSatF64S:u8 = 0x06;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const I64TruncSatF64U:u8 = 0x07;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const MemoryInit:u8 = 0x08;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const DataDrop:u8 = 0x09;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const MemoryCopy:u8 = 0x0a;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const MemoryFill:u8 = 0x0b;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const TableInit:u8 = 0x0c;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const ElemDrop:u8 = 0x0d;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const TableCopy:u8 = 0x0e;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const TableGrow:u8 = 0x0f;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const TableSize:u8 = 0x10;
    

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const TableFill:u8 = 0x11;
    
