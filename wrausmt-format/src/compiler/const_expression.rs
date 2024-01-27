use {
    super::validation::{ModuleContext, ValidationType},
    crate::{compiler::validation::KindResult, ValidationErrorKind},
    wrausmt_common::true_or::TrueOr,
    wrausmt_runtime::{
        instructions::opcodes,
        syntax::{
            types::{NumType, RefType, ValueType},
            CompiledExpr, FuncIndex, Index, Instruction, Operands, Resolved, UncompiledExpr,
        },
    },
};

macro_rules! instr {
    ($opcode:pat => $operands:pat) => {
        Instruction {
            opcode: $opcode,
            operands: $operands,
            ..
        }
    };
}

/// A separate emitter/validater for constant expressions.
/// Rather than including branching logic in the primary emitter/validator, it's
/// much clearer to create a parallel implementation here.
///
/// There are some different validation conditions that don't apply to the main
/// validator (imported globals only, const globals only, ref funcs can be
/// updated).
pub fn compile_const_expr(
    expr: &UncompiledExpr<Resolved>,
    module: &ModuleContext,
    funcrefs: &mut Vec<Index<Resolved, FuncIndex>>,
    expect_type: ValueType,
) -> KindResult<CompiledExpr> {
    let mut out = Vec::<u8>::new();
    let mut stack = Vec::<ValueType>::new();
    for instr in &expr.instr {
        validate_and_emit_instr(instr, &mut out, module, funcrefs, &mut stack)?;
    }

    (stack == [expect_type]).true_or(ValidationErrorKind::TypeMismatch {
        actual: ValidationType::Value(
            *stack
                .first()
                .ok_or(ValidationErrorKind::ValStackUnderflow)?,
        ),
        expect: ValidationType::Value(expect_type),
    })?;

    Ok(CompiledExpr {
        instr: out.into_boxed_slice(),
    })
}

fn validate_and_emit_instr(
    instr: &Instruction<Resolved>,
    out: &mut Vec<u8>,
    module: &ModuleContext,
    funcrefs: &mut Vec<Index<Resolved, FuncIndex>>,
    stack: &mut Vec<ValueType>,
) -> KindResult<()> {
    out.extend(instr.opcode.bytes());
    match instr {
        instr!(opcodes::I32_CONST => Operands::I32(v)) => {
            stack.push(NumType::I32.into());

            let bytes = &v.to_le_bytes()[..];
            out.extend(bytes);
        }
        instr!(opcodes::I64_CONST => Operands::I64(v)) => {
            stack.push(NumType::I64.into());

            let bytes = &v.to_le_bytes()[..];
            out.extend(bytes);
        }
        instr!(opcodes::F32_CONST => Operands::F32(v)) => {
            stack.push(NumType::F32.into());

            let bytes = &v.to_bits().to_le_bytes()[..];
            out.extend(bytes);
        }
        instr!(opcodes::F64_CONST => Operands::F64(v)) => {
            stack.push(NumType::F64.into());

            let bytes = &v.to_bits().to_le_bytes()[..];
            out.extend(bytes);
        }
        instr!(opcodes::REF_NULL => Operands::HeapType(ht)) => {
            stack.push((*ht).into());

            let htbyte = match ht {
                RefType::Func => 0x70,
                RefType::Extern => 0x6F,
            };
            out.push(htbyte);
        }
        instr!(opcodes::REF_FUNC => Operands::FuncIndex(fi)) => {
            ((fi.value() as usize) < module.funcs.len())
                .true_or(ValidationErrorKind::UnknownFunc)?;
            funcrefs.push(fi.clone());
            stack.push(RefType::Func.into());

            let bytes = &fi.value().to_le_bytes()[..];
            out.extend(bytes);
        }
        instr!(opcodes::GLOBAL_GET => Operands::GlobalIndex(gi)) => {
            let global = module
                .globals
                .get(gi.value() as usize)
                .ok_or(ValidationErrorKind::UnknownGlobal)?;
            (global.imported).true_or(ValidationErrorKind::InvalidConstantGlobal)?;
            (!global.globaltype.mutable)
                .true_or(ValidationErrorKind::InvalidConstantInstruction)?;
            stack.push(global.globaltype.valtype);

            let bytes = &gi.value().to_le_bytes()[..];
            out.extend(bytes);
        }
        _ => Err(ValidationErrorKind::InvalidConstantInstruction)?,
    };
    Ok(())
}
