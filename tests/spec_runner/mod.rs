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
    "address.wast",
    "align.wast",
    "binary-leb128.wast",
    "binary.wast",
    "br.wast",
    "br_if.wast",
    "br_table.wast",
    "comments.wast",
    "custom.wast",
    "const.wast",
    "data.wast",
    "fac.wast",
    "forward.wast",
    "float_literals.wast",
    "i32.wast",
    "i64.wast",
    "int_literals.wast",
    "load.wast",
    "nop.wast",
    "ref_null.wast",
    "return.wast",
    "select.wast",
    "store.wast",
    "switch.wast",
    "table.wast",
    "token.wast",
    "traps.wast",
    "type.wast",
    "unreachable.wast",
    "unreached-invalid.wast",
    "utf8-custom-section-id.wast",
    "utf8-import-field.wast",
    "utf8-import-module.wast",
    "utf8-invalid-encoding.wast",
];

#[test]
fn spec_tests_passing() -> Result<()> {
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
            let filename = &path.file_name().unwrap().to_str().unwrap();
            if ENABLED.iter().any(|n| n == filename) {
                println!("SKIP ALREADY SUCCEEDING {}", filename);
                continue;
            }
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
fn loopop() -> Result<()> {
    parse_and_run(
        "testdata/spec/loop.wast",
        runset_exclude!("as-compare-operand", "as-compare-operands", "nesting"),
    )
}

#[test]
fn callexclude() -> Result<()> {
    parse_and_run(
        "testdata/spec/call.wast",
        runset_exclude!("as-load-operand", "as-unary-operand", "as-convert-operand"),
    )
}
