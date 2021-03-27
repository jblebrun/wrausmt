#[derive(Debug)]
pub enum Inst {
    LocalGet(u32),
    Const32(u32),
    Add32
}
