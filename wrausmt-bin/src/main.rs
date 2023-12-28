use {wrausmt_format::loader::Loader, wrausmt_runtime::runtime::Runtime};

struct FlagsAndArgs {
    pub flags: Vec<(String, Option<String>)>,
    pub args:  Vec<String>,
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
        let module = if flags_and_args.flags.iter().any(|(f, _)| f == "--text") {
            runtime.load_wast(filename)
        } else {
            runtime.load_wasm(filename)
        };
        match module {
            Ok(_) => {}
            Err(e) => println!("Load failed: {}", e),
        }
    } else {
        println!(
            r#"Wrausmt Runner:

Provide a single filename.
By default it will be loaded as a binary wasm module if possible.
Provide the --text flag to parse as a text format module."#
        );
    }
}
