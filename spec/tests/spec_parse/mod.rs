use {
    spec::format::SpecParser,
    wrausmt::format::text::{lex::Tokenizer, parse::Parser},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_and_print(path: &str) -> Result<()> {
    let f = std::fs::File::open(path)?;

    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let spectest = parser.parse_spec_test()?;

    for cmd in &spectest.cmds {
        println!("cmd {:?}", cmd);
    }

    Ok(())
}

#[test]
fn spec_parse_call() -> Result<()> {
    parse_and_print("tests/data/call.wast")
}

#[test]
fn spec_parse_i32() -> Result<()> {
    parse_and_print("tests/data/i32.wast")
}

#[test]
fn spec_parse_align() -> Result<()> {
    parse_and_print("tests/data/align.wast")
}

#[test]
fn spec_parse_select() -> Result<()> {
    parse_and_print("tests/data/select.wast")
}
