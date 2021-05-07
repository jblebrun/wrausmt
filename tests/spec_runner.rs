use wrausmt::format::text::lex::Tokenizer;
use wrausmt::format::text::parse::Parser;
use wrausmt::spec::runner::run_spec_test;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_and_run(path: &str) -> Result<()> {
    let f = std::fs::File::open(path)?;

    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let spectest = parser.parse_spec_test()?;

    run_spec_test(spectest)?;

    Ok(())
}

#[test]
fn i32() -> Result<()> {
    parse_and_run("testdata/spec/i32.wast")
}

#[test]
fn i64() -> Result<()> {
    parse_and_run("testdata/spec/i64.wast")
}
