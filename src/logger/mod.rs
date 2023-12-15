pub trait Logger {
    fn log<F: Fn() -> String>(&self, tag: Tag, msg: F);
}

#[derive(Debug, Clone, Default)]
pub struct PrintLogger;

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
    Spec,

    Stack,
    LabelStack,
    ValStack,
    DumpStack,
    DumpLabelStack,
    DumpValStack,
}

impl Tag {
    fn enabled(&self) -> bool {
        matches!(self, Tag::Load | Tag::Spec | Tag::Host)
    }
}

impl Logger for PrintLogger {
    fn log<F: Fn() -> String>(&self, tag: Tag, msg: F) {
        if tag.enabled() {
            let msg = msg();
            println!("[{:?}] {}", tag, msg)
        }
    }
}
