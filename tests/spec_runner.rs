use wrausmt::format::text::lex::Tokenizer;
use wrausmt::format::text::parse::Parser;
use wrausmt::spec::runner::run_spec_test;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_and_run(path: &String) -> Result<()> {
    let f = std::fs::File::open(path)?;

    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let spectest = parser.parse_spec_test()?;

    run_spec_test(spectest)?;

    Ok(())
}

static ENABLED: &[&str] = &["comments.wast", "i32.wast", "i64.wast"];

#[test]
fn spec_tests() -> Result<()> {
    for item in ENABLED {
        let item = format!("testdata/spec/{}", item);
        match parse_and_run(&item) {
            Ok(()) => (),
            Err(e) => {
                println!("During {:?}", item);
                return Err(e);
            }
        }
    }
    Ok(())
}
