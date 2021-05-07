use std::path::Path;

use wrausmt::format::text::parse::Parser;
use wrausmt::spec::runner::run_spec_test;
use wrausmt::{format::text::lex::Tokenizer, spec::runner::RunSet};

use wrausmt::runset_exclude;
use wrausmt::runset_specific;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_and_run<S: AsRef<Path>>(path: S, runset: RunSet) -> Result<()> {
    let f = std::fs::File::open(path)?;

    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let spectest = parser.parse_spec_test()?;

    run_spec_test(spectest, runset)?;

    Ok(())
}

static ENABLED: &[&str] = &["comments.wast", "i32.wast", "i64.wast"];

#[test]
fn spec_tests() -> Result<()> {
    for item in ENABLED {
        let item = format!("testdata/spec/{}", item);
        match parse_and_run(&item, RunSet::All) {
            Ok(()) => (),
            Err(e) => {
                println!("During {:?}", item);
                return Err(e);
            }
        }
    }
    Ok(())
}

#[test]
fn callspecific() -> Result<()> {
    parse_and_run(
        "testdata/spec/call.wast",
        runset_specific!("type-i32", "type-i64"),
    )
}

#[test]
fn callexclude() -> Result<()> {
    parse_and_run(
        "testdata/spec/call.wast",
        runset_exclude!(
            "fac",
            "fac-acc",
            "fib",
            "as-select-first",
            "as-select-mid",
            "as-select-last",
            "as-br_if-first",
            "as-br_if-last",
            "as-br_table-first",
            "as-br_table-last",
            "as-store-first",
            "as-store-last",
            "as-memory.grow-value",
            "as-local.tee-value",
            "as-global.set-value",
            "as-load-operand",
            "as-unary-operand",
            "as-convert-operand"
        ),
    )
}
