use {
    tests::spec::{
        error::Result,
        loader::parse_and_run,
        runner::{RunConfig, RunSet},
    },
    wrausmt_format::ValidationMode,
};

const GLOBAL_FAILURES_TO_IGNORE: &[&str] = &[
    "alignment must not be larger than natural",
    "constant expression required",
    "duplicate export name",
    "global is immutable",
    "invalid result arity",
    "memory size must be at most 65536 pages (4GiB)",
    "multiple memories",
    "size minimum must not be greater than maximum",
    "start function",
    "type mismatch",
    "undeclared function reference",
    "unknown data segment",
    "unknown data segment 1",
    "unknown elem segment 0",
    "unknown elem segment 4",
    "unknown function",
    "unknown function 7",
    "unknown global",
    "unknown global 0",
    "unknown global 1",
    "unknown label",
    "unknown local",
    "unknown memory",
    "unknown memory 0",
    "unknown memory 1",
    "unknown table",
    "unknown table 0",
    "unknown type",
];
// To regenerate the spectest! lines below using the transform this macro
// expects: "".join(["spectest!(r#{});
// ".format(i.replace(".wast","").replace("-","_x_")) for i in
// sorted(os.listdir('testdata/spec'))])
macro_rules! spectest {
    ($name:ident; val:$vmode:expr; [$runset:expr]) => {
        #[test]
        fn $name() -> Result<()> {
            parse_and_run(
                format!("tests/spec/data/{}.wast", stringify!($name)[2..].replace("_x_", "-")),
                RunConfig {
                    runset: $runset,
                    validation_mode: $vmode,
                    failures_to_ignore: GLOBAL_FAILURES_TO_IGNORE
                }
            )
        }
    };
    ($name:ident; val:$vmode:expr) => {
        spectest!($name; val:$vmode; [RunSet::All]);
    };
    ($name:ident; [$runset:expr]) => {
        spectest!($name; val:ValidationMode::Warn; [$runset]);
    };
    ($name:ident) => { spectest!($name; val: ValidationMode::Warn; [RunSet::All]); };
}

#[allow(unused_macros)]
macro_rules! indices {
    ($($idx:literal),*) => {
        RunSet::SpecificIndex(vec![$($idx),*])
    }
}

spectest!(r#address);
spectest!(r#align; val:ValidationMode::Fail; [indices!(106)]);
spectest!(r#binary_x_leb128);
spectest!(r#binary);
spectest!(r#block);
spectest!(r#br);
spectest!(r#br_if);
spectest!(r#br_table);
spectest!(r#bulk);
spectest!(r#call);
spectest!(r#call_indirect);
spectest!(r#comments);
spectest!(r#const);
spectest!(r#conversions);
spectest!(r#custom);
spectest!(r#data);
spectest!(r#elem);
spectest!(r#endianness);
spectest!(r#exports);
spectest!(r#f32);
spectest!(r#f32_bitwise);
spectest!(r#f32_cmp);
spectest!(r#f64);
spectest!(r#f64_bitwise);
spectest!(r#f64_cmp);
spectest!(r#fac);
spectest!(r#float_exprs);
spectest!(r#float_literals);
spectest!(r#float_memory);
spectest!(r#float_misc);
spectest!(r#forward);
spectest!(r#func);
spectest!(r#func_ptrs);
spectest!(r#global);
spectest!(r#i32);
spectest!(r#i64);
spectest!(r#if);
spectest!(r#imports);
spectest!(r#inline_x_module);
spectest!(r#int_exprs);
spectest!(r#int_literals);
spectest!(r#labels);
spectest!(r#left_x_to_x_right);
spectest!(r#linking);
spectest!(r#load);
spectest!(r#local_get);
spectest!(r#local_set);
spectest!(r#local_tee);
spectest!(r#loop);
spectest!(r#memory);
spectest!(r#memory_copy);
spectest!(r#memory_fill);
spectest!(r#memory_grow);
spectest!(r#memory_init);
spectest!(r#memory_redundancy);
spectest!(r#memory_size);
spectest!(r#memory_trap);
spectest!(r#names);
spectest!(r#nop);
spectest!(r#obsolete_x_keywords);
spectest!(r#ref_func);
spectest!(r#ref_is_null);
spectest!(r#ref_null);
spectest!(r#return);
spectest!(r#select);
spectest!(r#skip_x_stack_x_guard_x_page);
spectest!(r#stack);
spectest!(r#start);
spectest!(r#store);
spectest!(r#switch);
spectest!(r#table_x_sub);
spectest!(r#table);
spectest!(r#table_copy);
spectest!(r#table_fill);
spectest!(r#table_get);
spectest!(r#table_grow);
spectest!(r#table_init);
spectest!(r#table_set);
spectest!(r#table_size);
spectest!(r#token);
spectest!(r#traps);
spectest!(r#type);
spectest!(r#unreachable);
spectest!(r#unreached_x_invalid);
spectest!(r#unreached_x_valid);
spectest!(r#unwind);
spectest!(r#utf8_x_custom_x_section_x_id);
spectest!(r#utf8_x_import_x_field);
spectest!(r#utf8_x_import_x_module);
spectest!(r#utf8_x_invalid_x_encoding);
