use wrausmt::{error::{Result, ResultFrom}, format::text::{lex::Tokenizer, parse::Parser}, spec::runner::run_spec_test};

fn parse_and_run(path: &str) -> Result<()> {
    let f = std::fs::File::open(path).wrap(&format!("opening file {}", path))?;

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
