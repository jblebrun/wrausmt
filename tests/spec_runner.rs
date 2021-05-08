use std::path::Path;

use wrausmt::format::text::parse::Parser;
use wrausmt::spec::runner::run_spec_test;
use wrausmt::{format::text::lex::Tokenizer, spec::runner::RunSet};

use wrausmt::runset_exclude;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_and_run<S: AsRef<Path>>(path: S, runset: RunSet) -> Result<()> {
    let f = std::fs::File::open(path)?;

    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let spectest = parser.parse_spec_test()?;

    run_spec_test(spectest, runset)?;

    Ok(())
}

static ENABLED: &[&str] = &[
    "br.wast",
    "br_table.wast",
    "comments.wast",
    "i32.wast",
    "i64.wast",
];

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
fn select() -> Result<()> {
    parse_and_run("testdata/spec/select.wast", RunSet::First(82))
}

#[test]
fn loopop() -> Result<()> {
    parse_and_run("testdata/spec/loop.wast", RunSet::First(13))
}

#[test]
fn nop() -> Result<()> {
    parse_and_run("testdata/spec/nop.wast", RunSet::First(50))
}

#[test]
fn br_if() -> Result<()> {
    parse_and_run("testdata/spec/br_if.wast", RunSet::First(20))
}

#[test]
fn callexclude() -> Result<()> {
    parse_and_run(
        "testdata/spec/call.wast",
        runset_exclude!(
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
