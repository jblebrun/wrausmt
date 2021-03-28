#[derive(Debug, Clone)]
pub enum Inst {
    LocalGet(u32),
    Const32(u32),
    Add32
}


