#[derive(Debug, Clone)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32
}

#[derive(Debug, Clone)]
pub enum Inst {
    // Control
    Nop,
    Block(u32),
    Loop(u32),
    If(u32)
    Br(u32),
    BrIf(u32),
    BrTable(u32),
    Return,
    Call(u32),
    CallIndirect(u32, u32),
    
    // Reference
    RefNull(u32),
    RefIsNull,
    RefFunc(u32),

    // Parametric
    Drop,
    Select,
    SelectT(Box<[u32]>), 

    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),

    I32_Const(u32),


    I32_Load(MemArg),
    I32_Store(MemArg),

    Add32,
    I32_Lt,
    I32_Gt,

    End
}


