use wrausmt::format::text::parse::Parser;
use wrausmt::format::text::lex::Tokenizer;
use wrausmt::error::{Result, ResultFrom};

fn parse_and_print(path: &str) -> Result<()> {
    let f = std::fs::File::open(path).wrap(&format!("opening file {}", path))?;

    let tokenizer = Tokenizer::new(f).wrap("tokenizer")?;
    let mut parser = Parser::new(tokenizer)?;
    let spectest = parser.parse_spec_test()?;

    for cmd in &spectest.cmds {
        println!("cmd {:?}", cmd);
    }
    
    Ok(())
}

#[test]
fn spec_parse_call() -> Result<()> {
    parse_and_print("testdata/spec/call.wast")
}

#[test]
fn spec_parse_i32() -> Result<()> {
    parse_and_print("testdata/spec/i32.wast")
}

#[test]
fn spec_parse_align() -> Result<()> {
    parse_and_print("testdata/spec/align.wast")
}
