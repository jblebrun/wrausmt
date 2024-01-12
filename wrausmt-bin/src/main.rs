use {
    wrausmt_format::{file_loader::FileLoader, ValidationMode},
    wrausmt_runtime::runtime::Runtime,
};

#[derive(Debug)]
struct FlagsAndArgs {
    pub flags: Vec<(String, Option<String>)>,
    pub args:  Vec<String>,
}

impl FlagsAndArgs {
    fn has_flag(&self, name: &str) -> bool {
        self.flags.iter().any(|(f, _)| f == name)
    }
}

impl FlagsAndArgs {
    fn new() -> Self {
        let mut flags: Vec<(String, Option<String>)> = Vec::new();
        let mut args: Vec<String> = Vec::new();
        let mut arg_iter = std::env::args();
        while let Some(arg) = arg_iter.next() {
            match arg {
                arg if arg.starts_with("--") => {
                    flags.push((arg, arg_iter.next()));
                }
                arg if arg.starts_with('-') => flags.push((arg[1..].to_owned(), None)),
                _ => args.push(arg),
            }
        }
        FlagsAndArgs { flags, args }
    }
}

fn main() {
    let flags_and_args = FlagsAndArgs::new();
    if let Some(filename) = flags_and_args.args.get(1) {
        let mut runtime = Runtime::new();
        let validation_mode = if flags_and_args.has_flag("no-validation") {
            ValidationMode::Warn
        } else {
            ValidationMode::Fail
        };
        let module = runtime.load_file_with_validation_mode(filename, validation_mode);
        match module {
            Ok(_) => {}
            Err(e) => println!("Load failed: {}", e),
        }
    } else {
        println!(
            r"Wrausmt Runner:

Provide a single filename.

It will be loaded as a binary file if it starts with the magic header, otherwise
it will be loaded as a text file."
        );
    }
}
