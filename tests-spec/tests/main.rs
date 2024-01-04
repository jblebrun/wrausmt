use spec::{error::Result, loader::parse_and_run, runner::RunSet};

// To regenerate the spectest! lines below using the transform this macro
// expects: "".join(["spectest!(r#{});
// ".format(i.replace(".wast","").replace("-","_x_")) for i in
// sorted(os.listdir('testdata/spec'))])
macro_rules! spectest {
    ($name:ident; [$runset:expr]) => {
        #[test]
        fn $name() -> Result<()> {
            parse_and_run(
                format!("tests/data/{}.wast", stringify!($name)[2..].replace("_x_", "-")),
                $runset,
            )
        }
    };
    ($name:ident) => { spectest!($name; [RunSet::All]); };
}

macro_rules! nomalformed {
    ($($failure:literal),*) => {
        RunSet::ExcludeFailure(vec![$($failure.into()),*])
    };
}

#[allow(unused_macros)]
macro_rules! indices {
    ($($idx:literal),*) => {
        RunSet::SpecificIndex(vec![$($idx),*])
    }
}

spectest!(r#address);
spectest!(r#align);
spectest!(r#binary_x_leb128);
spectest!(r#binary; [nomalformed!(
    "magic header not detected",
    "unexpected end",
    "unknown binary version",
    "malformed section id",
    "END opcode expected",
    "unexpected end of section or function",
    "section size mismatch",
    "illegal opcode",
    "zero byte expected",
    "integer too large",
    "too many locals",
    "function and code section have inconsistent lengths",
    "data count and data section have inconsistent lengths",
    "data count section required",
    "malformed reference type",
    "length out of bounds",
    "malformed import kind",
    "unexpected content after last section"
)]);
spectest!(r#block; [nomalformed!("mismatching label")]);
spectest!(r#br);
spectest!(r#br_if);
spectest!(r#br_table);
spectest!(r#bulk);
spectest!(r#call);
spectest!(r#call_indirect);
spectest!(r#comments);
spectest!(r#const; [nomalformed!("unknown operator", "constant out of range")]);
spectest!(r#conversions);
spectest!(r#custom; [nomalformed!(
    "unexpected end",
    "length out of bounds",
    "malformed section id",
    "function and code section have inconsistent lengths",
    "data count and data section have inconsistent lengths"
)]);
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
spectest!(r#float_literals; [nomalformed!("unknown operator", "constant out of range")]);
spectest!(r#float_memory);
spectest!(r#float_misc);
spectest!(r#forward);
spectest!(r#func; [nomalformed!(
    "duplicate func",
    "duplicate local",
    "unknown type"
)]);
spectest!(r#func_ptrs);
spectest!(r#global; [nomalformed!("malformed mutability", "duplicate global")]);
spectest!(r#i32);
spectest!(r#i64);
spectest!(r#if; [nomalformed!("mismatching label")]);
spectest!(r#imports; [nomalformed!("import after function", "import after global", "import after table", "import after memory")]);
spectest!(r#inline_x_module);
spectest!(r#int_exprs);
spectest!(r#int_literals; [nomalformed!("unknown operator")]);
spectest!(r#labels);
spectest!(r#left_x_to_x_right);
spectest!(r#linking);
spectest!(r#load);
spectest!(r#local_get);
spectest!(r#local_set);
spectest!(r#local_tee);
spectest!(r#loop; [nomalformed!("mismatching label")]);
spectest!(r#memory; [nomalformed!("i32 constant out of range", "duplicate memory")]);
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
spectest!(r#start; [nomalformed!("multiple start sections")]);
spectest!(r#store);
spectest!(r#switch);
spectest!(r#table_x_sub);
spectest!(r#table; [nomalformed!("i32 constant out of range", "duplicate table")]);
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
spectest!(r#utf8_x_custom_x_section_x_id; [nomalformed!("malformed UTF-8")]);
spectest!(r#utf8_x_import_x_field; [nomalformed!("malformed UTF-8")]);
spectest!(r#utf8_x_import_x_module; [nomalformed!("malformed UTF-8")]);
spectest!(r#utf8_x_invalid_x_encoding; [nomalformed!("malformed UTF-8")]);
