pub trait Logger {
    fn log(&self, tag: Tag, msg: impl Fn() -> String);
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
    SpecModule,
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
    fn log(&self, tag: Tag, msg: impl Fn() -> String) {
        if tag.enabled() {
            let msg = msg();
            println!("[{:?}] {}", tag, msg)
        }
    }
}
