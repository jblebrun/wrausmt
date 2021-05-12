use crate::{
    format::text::module_builder::ModuleBuilder,
    syntax::{self, GlobalField, Instruction, MemoryField, Resolved, TableField},
    types::{GlobalType, Limits, MemType, NumType, TableType, ValueType},
};

///(module
///  (global (export "global_i32") i32)
///  (global (export "global_i64") i64)
///  (global (export "global_f32") f32)
///  (global (export "global_f64") f64)
///
///  (table (export "table") 10 20 funcref)
///
///  (memory (export "memory") 1 2)
///
///  (func (export "print"))
///  (func (export "print_i32") (param i32))
///  (func (export "print_i64") (param i64))
///  (func (export "print_f32") (param f32))
///  (func (export "print_f64") (param f64))
///  (func (export "print_i32_f32") (param i32 f32))
///  (func (export "print_f64_f64") (param f64 f64))
///)
pub fn make_spectest_module() -> syntax::Module<Resolved> {
    let mut builder = ModuleBuilder::default();

    builder.add_globalfield(GlobalField {
        id: None,
        exports: vec!["global_i32".to_owned()],
        globaltype: GlobalType {
            mutable: false,
            valtype: ValueType::Num(NumType::I32),
        },
        init: syntax::Expr {
            instr: vec![Instruction::i32const(0)],
        },
    });

    builder.add_globalfield(GlobalField {
        id: None,
        exports: vec!["global_i64".to_owned()],
        globaltype: GlobalType {
            mutable: false,
            valtype: ValueType::Num(NumType::I64),
        },
        init: syntax::Expr {
            instr: vec![Instruction::i64const(0)],
        },
    });
    builder.add_globalfield(GlobalField {
        id: None,
        exports: vec!["global_f32".to_owned()],
        globaltype: GlobalType {
            mutable: false,
            valtype: ValueType::Num(NumType::F32),
        },
        init: syntax::Expr {
            instr: vec![Instruction::f32const(0f32)],
        },
    });

    builder.add_globalfield(GlobalField {
        id: None,
        exports: vec!["global_f64".to_owned()],
        globaltype: GlobalType {
            mutable: false,
            valtype: ValueType::Num(NumType::F64),
        },
        init: syntax::Expr {
            instr: vec![Instruction::f64const(0f64)],
        },
    });

    builder.add_tablefield(TableField {
        id: None,
        exports: vec!["table".to_owned()],
        tabletype: TableType {
            limits: Limits {
                lower: 10,
                upper: Some(20),
            },
            reftype: crate::types::RefType::Func,
        },
    });
    builder.add_memoryfield(MemoryField {
        id: None,
        exports: vec!["memory".to_owned()],
        memtype: MemType {
            limits: Limits {
                lower: 1,
                upper: Some(2),
            },
        },
        init: vec![],
    });

    builder.build().unwrap()
}
