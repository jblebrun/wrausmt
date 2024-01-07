#[derive(Debug)]
pub enum Tag {
    Activate,
    Enter,
    Flow,
    Load,
    Local,
    Mem,
    Op,
    Host,
    Stack,
    LabelStack,
    ValStack,
    DumpStack,
    DumpLabelStack,
    DumpValStack,
}

impl wrausmt_common::logger::Tag for Tag {
    fn enabled(&self) -> bool {
        matches!(self, Tag::Load | Tag::Host)
    }
}
