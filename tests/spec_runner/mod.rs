use std::{fs::File, path::Path, time::Instant};

use wrausmt::loader::Result as LoaderResult;
use wrausmt::spec::error::Result as SpecTestResult;
use wrausmt::spec::runner::SpecTestRunner;
use wrausmt::{format::text::lex::Tokenizer, spec::runner::RunSet};
use wrausmt::{format::text::parse::Parser, spec::format::SpecTestScript};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
#[allow(dead_code)]
enum FailMode {
    Parse,
    Run,
}

fn parse(f: &mut File) -> LoaderResult<SpecTestScript> {
    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let result = parser.parse_spec_test()?;
    Ok(result)
}

fn parse_and_run_for_result(mut f: File, runset: RunSet) -> SpecTestResult<()> {
    println!("\n\n*****  PARSING {:?} *****\n\n", f);
    let spectest = parse(&mut f)?;
    let start = Instant::now();
    println!("\n\n*****  RUNNING {:?} *****\n\n", f);
    let runner = SpecTestRunner::new();
    let result = runner.run_spec_test(spectest, runset);
    let finish = Instant::now();
    println!("TIMING {:?} IN {:?}", f, (finish - start));
    result
}

fn parse_and_run<S: std::fmt::Debug + AsRef<Path>>(
    path: S,
    runset: RunSet,
    mode: FailMode,
) -> Result<()> {
    let f = std::fs::File::open(&path)?;
    let result = parse_and_run_for_result(f, runset);
    let passingtext = match result {
        Ok(()) => "PASSING",
        _ => "FAILING",
    };
    println!("MODE {:?} {} ALL {:?}", mode, passingtext, path);
    match result {
        Err(e) => match mode {
            FailMode::Parse => match e {
                e if e.is_parse_error() => Err(Box::new(e)),
                _ => {
                    println!("{:?} -- SKIPPING PARSE MODE error {:?}", path, e);
                    Ok(())
                }
            },
            FailMode::Run => Err(Box::new(e)),
        },
        Ok(()) => Ok(()),
    }
}

#[test]
fn r#address() -> Result<()> {
    parse_and_run("testdata/spec/address.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#align() -> Result<()> {
    parse_and_run("testdata/spec/align.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#binary_leb128() -> Result<()> {
    parse_and_run(
        "testdata/spec/binary-leb128.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#binary() -> Result<()> {
    parse_and_run("testdata/spec/binary.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#block() -> Result<()> {
    parse_and_run("testdata/spec/block.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#br() -> Result<()> {
    parse_and_run("testdata/spec/br.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#br_if() -> Result<()> {
    parse_and_run("testdata/spec/br_if.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#br_table() -> Result<()> {
    parse_and_run("testdata/spec/br_table.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#bulk() -> Result<()> {
    parse_and_run("testdata/spec/bulk.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#call() -> Result<()> {
    parse_and_run("testdata/spec/call.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#call_indirect() -> Result<()> {
    parse_and_run(
        "testdata/spec/call_indirect.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#comments() -> Result<()> {
    parse_and_run("testdata/spec/comments.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#const() -> Result<()> {
    parse_and_run("testdata/spec/const.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#conversions() -> Result<()> {
    parse_and_run("testdata/spec/conversions.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#custom() -> Result<()> {
    parse_and_run("testdata/spec/custom.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#data() -> Result<()> {
    parse_and_run("testdata/spec/data.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#elem() -> Result<()> {
    parse_and_run("testdata/spec/elem.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#endianness() -> Result<()> {
    parse_and_run("testdata/spec/endianness.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#exports() -> Result<()> {
    parse_and_run("testdata/spec/exports.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#f32() -> Result<()> {
    parse_and_run("testdata/spec/f32.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#f32_bitwise() -> Result<()> {
    parse_and_run("testdata/spec/f32_bitwise.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#f32_cmp() -> Result<()> {
    parse_and_run("testdata/spec/f32_cmp.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#f64() -> Result<()> {
    parse_and_run("testdata/spec/f64.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#f64_bitwise() -> Result<()> {
    parse_and_run("testdata/spec/f64_bitwise.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#f64_cmp() -> Result<()> {
    parse_and_run("testdata/spec/f64_cmp.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#fac() -> Result<()> {
    parse_and_run("testdata/spec/fac.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#float_exprs() -> Result<()> {
    parse_and_run("testdata/spec/float_exprs.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#float_literals() -> Result<()> {
    parse_and_run(
        "testdata/spec/float_literals.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#float_memory() -> Result<()> {
    parse_and_run(
        "testdata/spec/float_memory.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#float_misc() -> Result<()> {
    parse_and_run("testdata/spec/float_misc.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#forward() -> Result<()> {
    parse_and_run("testdata/spec/forward.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#func() -> Result<()> {
    parse_and_run("testdata/spec/func.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#func_ptrs() -> Result<()> {
    parse_and_run("testdata/spec/func_ptrs.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#global() -> Result<()> {
    parse_and_run("testdata/spec/global.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#i32() -> Result<()> {
    parse_and_run("testdata/spec/i32.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#i64() -> Result<()> {
    parse_and_run("testdata/spec/i64.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#if() -> Result<()> {
    parse_and_run("testdata/spec/if.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#imports() -> Result<()> {
    parse_and_run("testdata/spec/imports.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#inline_module() -> Result<()> {
    parse_and_run(
        "testdata/spec/inline-module.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#int_exprs() -> Result<()> {
    parse_and_run("testdata/spec/int_exprs.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#int_literals() -> Result<()> {
    parse_and_run(
        "testdata/spec/int_literals.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#labels() -> Result<()> {
    parse_and_run("testdata/spec/labels.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#left_to_right() -> Result<()> {
    parse_and_run(
        "testdata/spec/left-to-right.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#linking() -> Result<()> {
    parse_and_run("testdata/spec/linking.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#load() -> Result<()> {
    parse_and_run("testdata/spec/load.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#local_get() -> Result<()> {
    parse_and_run("testdata/spec/local_get.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#local_set() -> Result<()> {
    parse_and_run("testdata/spec/local_set.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#local_tee() -> Result<()> {
    parse_and_run("testdata/spec/local_tee.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#loop() -> Result<()> {
    parse_and_run("testdata/spec/loop.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#memory() -> Result<()> {
    parse_and_run("testdata/spec/memory.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#memory_copy() -> Result<()> {
    parse_and_run("testdata/spec/memory_copy.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#memory_fill() -> Result<()> {
    parse_and_run("testdata/spec/memory_fill.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#memory_grow() -> Result<()> {
    parse_and_run("testdata/spec/memory_grow.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#memory_init() -> Result<()> {
    parse_and_run("testdata/spec/memory_init.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#memory_redundancy() -> Result<()> {
    parse_and_run(
        "testdata/spec/memory_redundancy.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#memory_size() -> Result<()> {
    parse_and_run("testdata/spec/memory_size.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#memory_trap() -> Result<()> {
    parse_and_run("testdata/spec/memory_trap.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#names() -> Result<()> {
    parse_and_run("testdata/spec/names.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#nop() -> Result<()> {
    parse_and_run("testdata/spec/nop.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#ref_func() -> Result<()> {
    parse_and_run("testdata/spec/ref_func.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#ref_is_null() -> Result<()> {
    parse_and_run("testdata/spec/ref_is_null.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#ref_null() -> Result<()> {
    parse_and_run("testdata/spec/ref_null.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#return() -> Result<()> {
    parse_and_run("testdata/spec/return.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#select() -> Result<()> {
    parse_and_run("testdata/spec/select.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#skip_stack_guard_page() -> Result<()> {
    parse_and_run(
        "testdata/spec/skip-stack-guard-page.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#stack() -> Result<()> {
    parse_and_run("testdata/spec/stack.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#start() -> Result<()> {
    parse_and_run("testdata/spec/start.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#store() -> Result<()> {
    parse_and_run("testdata/spec/store.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#switch() -> Result<()> {
    parse_and_run("testdata/spec/switch.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#table_sub() -> Result<()> {
    parse_and_run("testdata/spec/table-sub.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#table() -> Result<()> {
    parse_and_run("testdata/spec/table.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#table_copy() -> Result<()> {
    parse_and_run("testdata/spec/table_copy.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#table_fill() -> Result<()> {
    parse_and_run("testdata/spec/table_fill.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#table_get() -> Result<()> {
    parse_and_run("testdata/spec/table_get.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#table_grow() -> Result<()> {
    parse_and_run("testdata/spec/table_grow.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#table_init() -> Result<()> {
    parse_and_run("testdata/spec/table_init.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#table_set() -> Result<()> {
    parse_and_run("testdata/spec/table_set.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#table_size() -> Result<()> {
    parse_and_run("testdata/spec/table_size.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#token() -> Result<()> {
    parse_and_run("testdata/spec/token.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#traps() -> Result<()> {
    parse_and_run("testdata/spec/traps.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#type() -> Result<()> {
    parse_and_run("testdata/spec/type.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#unreachable() -> Result<()> {
    parse_and_run("testdata/spec/unreachable.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#unreached_invalid() -> Result<()> {
    parse_and_run(
        "testdata/spec/unreached-invalid.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#unwind() -> Result<()> {
    parse_and_run("testdata/spec/unwind.wast", RunSet::All, FailMode::Run)
}

#[test]
fn r#utf8_custom_section_id() -> Result<()> {
    parse_and_run(
        "testdata/spec/utf8-custom-section-id.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#utf8_import_field() -> Result<()> {
    parse_and_run(
        "testdata/spec/utf8-import-field.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#utf8_import_module() -> Result<()> {
    parse_and_run(
        "testdata/spec/utf8-import-module.wast",
        RunSet::All,
        FailMode::Run,
    )
}

#[test]
fn r#utf8_invalid_encoding() -> Result<()> {
    parse_and_run(
        "testdata/spec/utf8-invalid-encoding.wast",
        RunSet::All,
        FailMode::Run,
    )
}
