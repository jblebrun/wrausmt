use std::{path::Path, time::Instant};

use wrausmt::format::text::parse::Parser;
use wrausmt::spec::runner::run_spec_test;
use wrausmt::{format::text::lex::Tokenizer, spec::runner::RunSet};

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
    "block.wast",
    "br.wast",
    "br_if.wast",
    "br_table.wast",
    "call.wast",
    "call_indirect.wast",
    "comments.wast",
    "custom.wast",
    "const.wast",
    "data.wast",
    "endianness.wast",
    "f32.wast",
    "f32_bitwise.wast",
    "f32_cmp.wast",
    "f64.wast",
    "f64_bitwise.wast",
    "f64_cmp.wast",
    "fac.wast",
    "forward.wast",
    "float_exprs.wast",
    "float_literals.wast",
    "float_memory.wast",
    "func.wast",
    "func_ptrs.wast",
    "i32.wast",
    "i64.wast",
    "if.wast",
    "int_exprs.wast",
    "int_literals.wast",
    "left-to-right.wast",
    "labels.wast",
    "load.wast",
    "local_get.wast",
    "local_set.wast",
    "local_tee.wast",
    "loop.wast",
    "memory_copy.wast",
    "memory_fill.wast",
    "memory_grow.wast",
    "memory_init.wast",
    "memory_redundancy.wast",
    "memory_size.wast",
    "names.wast",
    "nop.wast",
    "ref_func.wast",
    "ref_null.wast",
    "ref_is_null.wast",
    "return.wast",
    "select.wast",
    "skip-stack-guard-page.wast",
    "store.wast",
    "switch.wast",
    "table.wast",
    "table_copy.wast",
    "table_fill.wast",
    "table_get.wast",
    "table_grow.wast",
    "table_init.wast",
    "table_set.wast",
    "table_size.wast",
    "table-sub.wast",
    "token.wast",
    "traps.wast",
    "type.wast",
    "unreachable.wast",
    "unreached-invalid.wast",
    "unwind.wast",
    "utf8-custom-section-id.wast",
    "utf8-import-field.wast",
    "utf8-import-module.wast",
    "utf8-invalid-encoding.wast",
];

#[test]
fn spec_tests_passing() -> Result<()> {
    for item in ENABLED {
        let item = format!("testdata/spec/{}", item);
        let start = Instant::now();
        match parse_and_run(&item, RunSet::All) {
            Ok(()) => (),
            Err(e) => {
                println!("During {:?}", item);
                return Err(e);
            }
        }
        let finish = Instant::now();
        println!("{} FINISHED IN {:?}", item, (finish - start));
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
            let start = Instant::now();
            println!("\n\n*****  RUNNING {:?} *****\n\n", path);
            match parse_and_run(&path, RunSet::All) {
                Ok(()) => {
                    println!("Tests succeeded {:?}", path);
                }
                Err(e) => {
                    println!("During {:?}", path);
                    println!("{:?}", e);
                }
            }
            let finish = Instant::now();
            println!("TIMING {} IN {:?}", filename, (finish - start));
        }
    }

    Ok(())
}

#[test]
fn memcopy() -> Result<()> {
    parse_and_run("testdata/spec/memory_copy.wast", RunSet::All)
}
