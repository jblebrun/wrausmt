use std::path::Path;

use wrausmt::spec::runner::run_spec_test;
use wrausmt::{format::text::lex::Tokenizer, spec::runner::RunSet};
use wrausmt::{format::text::parse::Parser, runset_specific};

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
    "br_if.wast",
    "br_table.wast",
    "comments.wast",
    "i32.wast",
    "i64.wast",
    "nop.wast",
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
fn spec_tests_all_run_ignore_failure() -> Result<()> {
    for entry in std::fs::read_dir("testdata/spec")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            println!("RUNNING {:?}", path);
            match parse_and_run(&path, RunSet::All) {
                Ok(()) => {
                    println!("Tests succeeded {:?}", path);
                }
                Err(e) => {
                    println!("{:?} During {:?}", e, path);
                }
            }
        }
    }

    Ok(())
}

#[test]
fn select() -> Result<()> {
    parse_and_run(
        "testdata/spec/select.wast",
        runset_exclude!("as-convert-operand"),
    )
}

#[test]
fn loopop() -> Result<()> {
    parse_and_run(
        "testdata/spec/loop.wast",
        runset_exclude!(
            "as-compare-operand",
            "as-compare-operands",
            "break-value",
            "break-multi-value",
            "param-break",
            "params-break",
            "params-id-break",
            "while",
            "for",
            "nesting"
        ),
    )
}

#[test]
fn callexclude() -> Result<()> {
    parse_and_run(
        "testdata/spec/call.wast",
        runset_exclude!("as-load-operand", "as-unary-operand", "as-convert-operand"),
    )
}

#[test]
fn callspecific() -> Result<()> {
    parse_and_run("testdata/spec/call.wast", runset_specific!("odd"))
}
