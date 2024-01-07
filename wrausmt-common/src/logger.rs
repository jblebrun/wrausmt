/// A very simple logging interface.
///
/// Logging statements are enabled/disabled via [Tag]s.
pub trait Logger {
    fn log(&self, tag: impl Tag, msg: impl Fn() -> String);
}

/// A logging implementation that just writes to stdout.
#[derive(Debug, Clone, Default)]
pub struct PrintLogger;

impl Logger for PrintLogger {
    fn log(&self, tag: impl Tag, msg: impl Fn() -> String) {
        if tag.enabled() {
            let msg = msg();
            println!("[{:?}] {}", tag, msg)
        }
    }
}

// Usually a [Tag] is implemented for an enum struct containing the tags needed
// for a particular crate/module.
// A separate method on the enum itself implements "enabled".
// Logging configuration is only available at compile time.
pub trait Tag: core::fmt::Debug {
    fn enabled(&self) -> bool;
}
