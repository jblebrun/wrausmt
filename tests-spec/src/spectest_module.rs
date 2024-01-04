use {
    wrausmt_format::text::{module_builder::ModuleBuilder, resolve::Result},
    wrausmt_runtime::syntax::{
        self,
        types::{GlobalType, Limits, MemType, NumType, TableType, ValueType},
        FParam, FuncField, FunctionType, GlobalField, Instruction, MemoryField, Resolved,
        TableField, TypeUse, Unresolved,
    },
};

/// (module
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
/// )
pub fn make_spectest_module() -> Result<syntax::Module<Resolved>> {
    let mut builder = ModuleBuilder::default();

    builder.add_globalfield(GlobalField {
        id:         None,
        exports:    vec!["global_i32".into()],
        globaltype: GlobalType {
            mutable: false,
            valtype: ValueType::Num(NumType::I32),
        },
        init:       syntax::Expr {
            instr: vec![Instruction::i32const(666)],
        },
    });

    builder.add_globalfield(GlobalField {
        id:         None,
        exports:    vec!["global_i64".into()],
        globaltype: GlobalType {
            mutable: false,
            valtype: ValueType::Num(NumType::I64),
        },
        init:       syntax::Expr {
            instr: vec![Instruction::i64const(666u64)],
        },
    });
    builder.add_globalfield(GlobalField {
        id:         None,
        exports:    vec!["global_f32".into()],
        globaltype: GlobalType {
            mutable: false,
            valtype: ValueType::Num(NumType::F32),
        },
        init:       syntax::Expr {
            instr: vec![Instruction::f32const(666f32)],
        },
    });

    builder.add_globalfield(GlobalField {
        id:         None,
        exports:    vec!["global_f64".into()],
        globaltype: GlobalType {
            mutable: false,
            valtype: ValueType::Num(NumType::F64),
        },
        init:       syntax::Expr {
            instr: vec![Instruction::f64const(666f64)],
        },
    });

    builder.add_tablefield(TableField {
        id:        None,
        exports:   vec!["table".into()],
        tabletype: TableType {
            limits:  Limits {
                lower: 10,
                upper: Some(20),
            },
            reftype: wrausmt_runtime::syntax::types::RefType::Func,
        },
    });
    builder.add_memoryfield(MemoryField {
        id:      None,
        exports: vec!["memory".into()],
        memtype: MemType {
            limits: Limits {
                lower: 1,
                upper: Some(2),
            },
        },
    });

    builder.add_funcfield(mkfunc("print", vec![]))?;

    builder.add_funcfield(mkfunc("print_i32", vec![FParam {
        id:        None,
        valuetype: ValueType::Num(NumType::I32),
    }]))?;

    builder.add_funcfield(mkfunc("print_i64", vec![FParam {
        id:        None,
        valuetype: ValueType::Num(NumType::I64),
    }]))?;

    builder.add_funcfield(mkfunc("print_f32", vec![FParam {
        id:        None,
        valuetype: ValueType::Num(NumType::F32),
    }]))?;

    builder.add_funcfield(mkfunc("print_f64", vec![FParam {
        id:        None,
        valuetype: ValueType::Num(NumType::F64),
    }]))?;

    builder.add_funcfield(mkfunc("print_i32_f32", vec![
        FParam {
            id:        None,
            valuetype: ValueType::Num(NumType::I32),
        },
        FParam {
            id:        None,
            valuetype: ValueType::Num(NumType::F32),
        },
    ]))?;

    builder.add_funcfield(mkfunc("print_f64_f64", vec![
        FParam {
            id:        None,
            valuetype: ValueType::Num(NumType::F64),
        },
        FParam {
            id:        None,
            valuetype: ValueType::Num(NumType::F64),
        },
    ]))?;

    builder.build()
}

fn mkfunc(name: impl Into<String>, params: Vec<FParam>) -> FuncField<Unresolved> {
    FuncField {
        exports: vec![name.into()],
        typeuse: TypeUse::AnonymousInline(FunctionType {
            params,
            results: vec![],
        }),
        ..FuncField::default()
    }
}
