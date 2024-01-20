use {
    super::{FunctionType, Result, Validation, ValidationError, ValidationMode},
    crate::compiler::validation::ValidationType,
    wrausmt_common::true_or::TrueOr,
    wrausmt_runtime::{
        instructions::opcodes,
        syntax::{
            types::{NumType, RefType, ValueType},
            Index, Instruction, LocalIndex, Operands, Resolved, TypeUse,
        },
    },
};

// Conveniences for implementing the checks below.
const I32: ValueType = ValueType::Num(NumType::I32);
const I64: ValueType = ValueType::Num(NumType::I64);
const F32: ValueType = ValueType::Num(NumType::F32);
const F64: ValueType = ValueType::Num(NumType::F64);
const FUNC: ValueType = ValueType::Ref(RefType::Func);

macro_rules! instr {
    ($opcode:pat) => {
        Instruction {
            opcode: $opcode,
            operands: Operands::None,
            ..
        }
    };
    ($opcode:pat => $operands:pat) => {
        Instruction {
            opcode: $opcode,
            operands: $operands,
            ..
        }
    };
}

macro_rules! meminstr {
    ($opcode:pat, align: $a:ident) => {
        instr!($opcode => Operands::Memargs($a, _))
    }
}

impl<'a> Validation<'a> {
    /// Validate one instruction. The returned error will respect the
    /// [`ValidationMode`] provided at creation.
    pub fn handle_instr(&mut self, instr: &Instruction<Resolved>) -> Result<()> {
        self.error_for_mode(|s| s.validation_result(instr))
    }

    fn error_for_mode(&mut self, op: impl Fn(&mut Self) -> Result<()>) -> Result<()> {
        match (self.mode, op(self)) {
            (_, Ok(())) => Ok(()),
            (ValidationMode::Warn, Err(e)) => {
                println!("WARNING: Validation Failed: {e:?}");
                Ok(())
            }
            (ValidationMode::Fail, r) => r,
            (ValidationMode::Panic, Err(e)) => {
                panic!("Validation failed: {e:?}")
            }
        }
    }

    fn constop(&mut self, o: ValueType) -> Result<()> {
        self.stacks.push_val(o);
        Ok(())
    }

    fn unop(&mut self, i: ValueType, o: ValueType) -> Result<()> {
        self.stacks.pop_val(i)?;
        self.stacks.push_val(o);
        Ok(())
    }

    fn binop(&mut self, a: ValueType, b: ValueType, o: ValueType) -> Result<()> {
        self.stacks.pop_val(a)?;
        self.stacks.pop_val(b)?;
        self.stacks.push_val(o);
        Ok(())
    }

    fn loadop(
        &mut self,
        i: ValueType,
        o: ValueType,
        alignment: u32,
        natural_alignment: u32,
    ) -> Result<()> {
        (alignment <= natural_alignment).true_or(ValidationError::AlignmentTooLarge(alignment))?;
        self.stacks.pop_val(i)?;
        self.stacks.push_val(o);
        Ok(())
    }

    fn storeop(
        &mut self,
        v: ValueType,
        a: ValueType,
        alignment: u32,
        natural_alignment: u32,
    ) -> Result<()> {
        (alignment <= natural_alignment).true_or(ValidationError::AlignmentTooLarge(alignment))?;
        self.stacks.pop_val(v)?;
        self.stacks.pop_val(a)?;
        Ok(())
    }

    fn function_type_for_typeuse(&self, typeuse: &TypeUse<Resolved>) -> FunctionType {
        if typeuse.index().value() == 0x040 {
            FunctionType::default()
        } else {
            self.module.types[typeuse.index().value() as usize].clone()
        }
    }

    fn local_type(&self, idx: &Index<Resolved, LocalIndex>) -> Result<ValueType> {
        self.localtypes
            .get(idx.value() as usize)
            .ok_or(ValidationError::UnknownLocal(idx.clone()))
            .copied()
    }

    fn validation_result(&mut self, instr: &Instruction<Resolved>) -> Result<()> {
        println!("VALIDATION {instr:?}");
        match instr {
            instr!(opcodes::UNREACHABLE) => self.stacks.unreachable(),
            instr!(opcodes::NOP) => Ok(()),

            instr!(opcodes::BLOCK => Operands::Block(_, typeuse, ..)) => {
                let ft = self.function_type_for_typeuse(typeuse);
                self.stacks.pop_vals(&ft.params)?;
                self.stacks.push_ctrl(opcodes::BLOCK, ft.params, ft.results);
                Ok(())
            }

            instr!(opcodes::LOOP => Operands::Block(_, typeuse, ..)) => {
                let ft = self.function_type_for_typeuse(typeuse);
                self.stacks.pop_vals(&ft.params)?;
                self.stacks.push_ctrl(opcodes::LOOP, ft.params, ft.results);
                Ok(())
            }

            instr!(opcodes::IF => Operands::If(_, typeuse, ..)) => {
                let ft = self.function_type_for_typeuse(typeuse);
                self.stacks.pop_val(I32)?;
                self.stacks.pop_vals(&ft.params)?;
                self.stacks.push_ctrl(opcodes::IF, ft.params, ft.results);
                Ok(())
            }

            instr!(opcodes::ELSE) => {
                let frame = self.stacks.pop_ctrl()?;
                (frame.opcode == opcodes::IF).true_or(ValidationError::OpcodeMismatch)?;
                self.stacks
                    .push_ctrl(frame.opcode, frame.start_types, frame.end_types);
                Ok(())
            }

            instr!(opcodes::END) => {
                let frame = self.stacks.pop_ctrl()?;
                self.stacks.push_vals(&frame.end_types);
                Ok(())
            }

            // 0x1A
            instr!(opcodes::DROP) => {
                self.stacks.drop_val()?;
                Ok(())
            }

            instr!(opcodes::BR => Operands::LabelIndex(idx)) => {
                self.stacks.pop_label_types(idx)?;
                self.stacks.unreachable()?;
                Ok(())
            }

            instr!(opcodes::BR_IF => Operands::LabelIndex(idx)) => {
                self.stacks.pop_val(I32)?;
                self.stacks.pop_label_types(idx)?;
                self.stacks.push_label_types(idx)?;
                self.stacks.unreachable()?;
                Ok(())
            }

            instr!(opcodes::BR_TABLE => Operands::BrTable(idxes, last)) => {
                self.stacks.pop_val(I32)?;
                let default_arity = self.stacks.label_arity(last)?;
                for idx in idxes {
                    let break_arity = self.stacks.label_arity(idx)?;
                    (break_arity == default_arity).true_or(ValidationError::BreakTypeMismatch)?;
                    self.stacks.pop_label_types(idx)?;
                    self.stacks.push_label_types(idx)?;
                }
                self.stacks.pop_label_types(last)?;
                self.stacks.unreachable()?;
                Ok(())
            }

            // The return instruction is a shortcut for an unconditional branch
            // to the outermost block, which implicitly is the body of the
            // current function.
            instr!(opcodes::RETURN) => {
                self.stacks.pop_return_types()?;
                self.stacks.unreachable()?;
                Ok(())
            }

            instr!(opcodes::CALL => Operands::FuncIndex(idx)) => {
                let ft = &self
                    .module
                    .funcs
                    .get(idx.value() as usize)
                    .ok_or(ValidationError::UnknownFunc)?;
                self.stacks.pop_vals(&ft.params)?;
                self.stacks.push_vals(&ft.results);
                Ok(())
            }

            instr!(opcodes::CALL_INDIRECT => Operands::CallIndirect(tabidx, typeuse)) => {
                self.module
                    .tables
                    .get(tabidx.value() as usize)
                    .map(|t| t.reftype == RefType::Func)
                    .ok_or(ValidationError::UnknownTable)?
                    .true_or(ValidationError::WrongTableType)?;
                let ft = self
                    .module
                    .types
                    .get(typeuse.index().value() as usize)
                    .ok_or(ValidationError::UnknownType)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_vals(&ft.params)?;
                self.stacks.push_vals(&ft.results);
                Ok(())
            }

            // Note: the binary 0x1b and 0x1c both turn into SELECT,
            // with opcode 0x1b, but different operand types.
            instr!(opcodes::SELECT) => {
                self.stacks.pop_val(I32)?;
                let n1 = self.stacks.pop_num()?;
                let n2 = self.stacks.pop_num()?;
                (n1 == n2).true_or(ValidationError::TypeMismatch {
                    actual: ValidationType::Value(n2.into()),
                    expect: ValidationType::Value(n1.into()),
                })?;
                self.stacks.push_val(n1.into());
                Ok(())
            }
            instr!(opcodes::SELECT => Operands::SelectT(t)) => {
                (t.len() == 1).true_or(ValidationError::UnsupportedSelect)?;
                let vt = t[0].valuetype;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(vt)?;
                self.stacks.pop_val(vt)?;
                self.stacks.push_val(vt);
                Ok(())
            }

            // 0x20
            instr!(opcodes::LOCAL_GET => Operands::LocalIndex(idx)) => {
                self.stacks.push_val(self.local_type(idx)?);
                Ok(())
            }
            instr!(opcodes::LOCAL_SET => Operands::LocalIndex(idx)) => {
                self.stacks.pop_val(self.local_type(idx)?)?;
                Ok(())
            }
            instr!(opcodes::LOCAL_TEE => Operands::LocalIndex(idx)) => {
                let ty = self.local_type(idx)?;
                self.stacks.pop_val(ty)?;
                self.stacks.push_val(ty);
                Ok(())
            }
            // 0x28
            meminstr!(opcodes::I32_LOAD, align: a) => self.loadop(I32, I32, *a, 4),
            meminstr!(opcodes::I64_LOAD, align: a) => self.loadop(I32, I64, *a, 8),
            meminstr!(opcodes::F32_LOAD, align: a) => self.loadop(I32, F32, *a, 4),
            meminstr!(opcodes::F64_LOAD, align: a) => self.loadop(I32, F64, *a, 8),
            meminstr!(opcodes::I32_LOAD8_S, align: a) => self.loadop(I32, I32, *a, 1),
            meminstr!(opcodes::I32_LOAD8_U, align: a) => self.loadop(I32, I32, *a, 1),
            meminstr!(opcodes::I32_LOAD16_S, align: a) => self.loadop(I32, I32, *a, 2),
            meminstr!(opcodes::I32_LOAD16_U, align: a) => self.loadop(I32, I32, *a, 2),
            meminstr!(opcodes::I64_LOAD8_S , align: a) => self.loadop(I32, I64, *a, 1),
            meminstr!(opcodes::I64_LOAD8_U , align: a) => self.loadop(I32, I64, *a, 1),
            meminstr!(opcodes::I64_LOAD16_S , align: a) => self.loadop(I32, I64, *a, 2),
            meminstr!(opcodes::I64_LOAD16_U , align: a) => self.loadop(I32, I64, *a, 2),
            meminstr!(opcodes::I64_LOAD32_S , align: a) => self.loadop(I32, I64, *a, 4),
            meminstr!(opcodes::I64_LOAD32_U , align: a) => self.loadop(I32, I64, *a, 4),

            // 0x36
            meminstr!(opcodes::I32_STORE, align: a) => self.storeop(I32, I32, *a, 4),
            meminstr!(opcodes::I64_STORE, align: a) => self.storeop(I64, I32, *a, 8),
            meminstr!(opcodes::F32_STORE, align: a) => self.storeop(F32, I32, *a, 4),
            meminstr!(opcodes::F64_STORE, align: a) => self.storeop(F64, I32, *a, 8),
            meminstr!(opcodes::I32_STORE8, align: a) => self.storeop(I32, I32, *a, 1),
            meminstr!(opcodes::I32_STORE16, align: a) => self.storeop(I32, I32, *a, 2),
            meminstr!(opcodes::I64_STORE8, align: a) => self.storeop(I64, I32, *a, 1),
            meminstr!(opcodes::I64_STORE16, align: a) => self.storeop(I64, I32, *a, 2),
            meminstr!(opcodes::I64_STORE32, align: a) => self.storeop(I64, I32, *a, 4),

            instr!(opcodes::MEMORY_SIZE) => {
                (!self.module.mems.is_empty()).true_or(ValidationError::UnknownMemory)?;
                self.stacks.push_val(I32);
                Ok(())
            }
            instr!(opcodes::MEMORY_GROW) => {
                (!self.module.mems.is_empty()).true_or(ValidationError::UnknownMemory)?;
                self.stacks.pop_val(I32)?;
                self.stacks.push_val(I32);
                Ok(())
            }
            // 0x41
            instr!(opcodes::I32_CONST => Operands::I32(_)) => self.constop(I32),
            instr!(opcodes::I64_CONST => Operands::I64(_)) => self.constop(I64),
            instr!(opcodes::F32_CONST => Operands::F32(_)) => self.constop(F32),
            instr!(opcodes::F64_CONST => Operands::F64(_)) => self.constop(F64),

            // 0x45
            instr!(opcodes::I32_EQZ) => self.unop(I32, I32),
            instr!(opcodes::I32_EQ) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_NE) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_LT_S) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_LT_U) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_GT_S) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_GT_U) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_LE_S) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_LE_U) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_GE_S) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_GE_U) => self.binop(I32, I32, I32),

            // 0x50
            instr!(opcodes::I64_EQZ) => self.unop(I64, I32),
            instr!(opcodes::I64_EQ) => self.binop(I64, I64, I32),
            instr!(opcodes::I64_NE) => self.binop(I64, I64, I32),
            instr!(opcodes::I64_LT_S) => self.binop(I64, I64, I32),
            instr!(opcodes::I64_LT_U) => self.binop(I64, I64, I32),
            instr!(opcodes::I64_GT_S) => self.binop(I64, I64, I32),
            instr!(opcodes::I64_GT_U) => self.binop(I64, I64, I32),
            instr!(opcodes::I64_LE_S) => self.binop(I64, I64, I32),
            instr!(opcodes::I64_LE_U) => self.binop(I64, I64, I32),
            instr!(opcodes::I64_GE_S) => self.binop(I64, I64, I32),
            instr!(opcodes::I64_GE_U) => self.binop(I64, I64, I32),

            // 0x5B
            instr!(opcodes::F32_EQ) => self.binop(F32, F32, I32),
            instr!(opcodes::F32_NE) => self.binop(F32, F32, I32),
            instr!(opcodes::F32_LT) => self.binop(F32, F32, I32),
            instr!(opcodes::F32_GT) => self.binop(F32, F32, I32),
            instr!(opcodes::F32_LE) => self.binop(F32, F32, I32),
            instr!(opcodes::F32_GE) => self.binop(F32, F32, I32),

            // 0x61
            instr!(opcodes::F64_EQ) => self.binop(F64, F64, I32),
            instr!(opcodes::F64_NE) => self.binop(F64, F64, I32),
            instr!(opcodes::F64_LT) => self.binop(F64, F64, I32),
            instr!(opcodes::F64_GT) => self.binop(F64, F64, I32),
            instr!(opcodes::F64_LE) => self.binop(F64, F64, I32),
            instr!(opcodes::F64_GE) => self.binop(F64, F64, I32),

            // 0x67
            instr!(opcodes::I32_CLZ) => self.unop(I32, I32),
            instr!(opcodes::I32_CTZ) => self.unop(I32, I32),
            instr!(opcodes::I32_POPCNT) => self.unop(I32, I32),
            instr!(opcodes::I32_ADD) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_SUB) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_MUL) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_DIV_S) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_DIV_U) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_REM_S) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_REM_U) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_AND) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_OR) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_XOR) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_SHL) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_SHR_S) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_SHR_U) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_ROTL) => self.binop(I32, I32, I32),
            instr!(opcodes::I32_ROTR) => self.binop(I32, I32, I32),

            // 0x79
            instr!(opcodes::I64_CLZ) => self.unop(I64, I64),
            instr!(opcodes::I64_CTZ) => self.unop(I64, I64),
            instr!(opcodes::I64_POPCNT) => self.unop(I64, I64),
            instr!(opcodes::I64_ADD) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_SUB) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_MUL) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_DIV_S) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_DIV_U) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_REM_S) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_REM_U) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_AND) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_OR) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_XOR) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_SHL) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_SHR_S) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_SHR_U) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_ROTL) => self.binop(I64, I64, I64),
            instr!(opcodes::I64_ROTR) => self.binop(I64, I64, I64),

            // 0x8B
            instr!(opcodes::F32_ABS) => self.unop(F32, F32),
            instr!(opcodes::F32_NEG) => self.unop(F32, F32),
            instr!(opcodes::F32_CEIL) => self.unop(F32, F32),
            instr!(opcodes::F32_FLOOR) => self.unop(F32, F32),
            instr!(opcodes::F32_TRUNC) => self.unop(F32, F32),
            instr!(opcodes::F32_NEAREST) => self.unop(F32, F32),
            instr!(opcodes::F32_SQRT) => self.unop(F32, F32),

            // 0x92
            instr!(opcodes::F32_ADD) => self.binop(F32, F32, F32),
            instr!(opcodes::F32_SUB) => self.binop(F32, F32, F32),
            instr!(opcodes::F32_MUL) => self.binop(F32, F32, F32),
            instr!(opcodes::F32_DIV) => self.binop(F32, F32, F32),
            instr!(opcodes::F32_MIN) => self.binop(F32, F32, F32),
            instr!(opcodes::F32_MAX) => self.binop(F32, F32, F32),
            instr!(opcodes::F32_COPYSIGN) => self.binop(F32, F32, F32),

            // 0x99
            instr!(opcodes::F64_ABS) => self.unop(F64, F64),
            instr!(opcodes::F64_NEG) => self.unop(F64, F64),
            instr!(opcodes::F64_CEIL) => self.unop(F64, F64),
            instr!(opcodes::F64_FLOOR) => self.unop(F64, F64),
            instr!(opcodes::F64_TRUNC) => self.unop(F64, F64),
            instr!(opcodes::F64_NEAREST) => self.unop(F64, F64),
            instr!(opcodes::F64_SQRT) => self.unop(F64, F64),

            // 0xA0
            instr!(opcodes::F64_ADD) => self.binop(F64, F64, F64),
            instr!(opcodes::F64_SUB) => self.binop(F64, F64, F64),
            instr!(opcodes::F64_MUL) => self.binop(F64, F64, F64),
            instr!(opcodes::F64_DIV) => self.binop(F64, F64, F64),
            instr!(opcodes::F64_MIN) => self.binop(F64, F64, F64),
            instr!(opcodes::F64_MAX) => self.binop(F64, F64, F64),
            instr!(opcodes::F64_COPYSIGN) => self.binop(F64, F64, F64),

            // 0xA7
            instr!(opcodes::I32_WRAP_I64) => self.unop(I64, I32),
            instr!(opcodes::I32_TRUNC_F32_S) => self.unop(F32, I32),
            instr!(opcodes::I32_TRUNC_F32_U) => self.unop(F32, I32),
            instr!(opcodes::I32_TRUNC_F64_S) => self.unop(F64, I32),
            instr!(opcodes::I32_TRUNC_F64_U) => self.unop(F64, I32),

            // 0xAC
            instr!(opcodes::I64_EXTEND_I32_S) => self.unop(I32, I64),
            instr!(opcodes::I64_EXTEND_I32_U) => self.unop(I32, I64),
            instr!(opcodes::I64_TRUNC_F32_S) => self.unop(F32, I64),
            instr!(opcodes::I64_TRUNC_F32_U) => self.unop(F32, I64),
            instr!(opcodes::I64_TRUNC_F64_S) => self.unop(F64, I64),
            instr!(opcodes::I64_TRUNC_F64_U) => self.unop(F64, I64),

            // 0xB2
            instr!(opcodes::F32_CONVERT_I32_S) => self.unop(I32, F32),
            instr!(opcodes::F32_CONVERT_I32_U) => self.unop(I32, F32),
            instr!(opcodes::F32_CONVERT_I64_S) => self.unop(I32, F32),
            instr!(opcodes::F32_CONVERT_I64_U) => self.unop(I32, F32),
            instr!(opcodes::F32_DEMOTE_F64) => self.unop(F64, F32),

            // 0xB7
            instr!(opcodes::F64_CONVERT_I32_S) => self.unop(I32, F64),
            instr!(opcodes::F64_CONVERT_I32_U) => self.unop(I32, F64),
            instr!(opcodes::F64_CONVERT_I64_S) => self.unop(I32, F64),
            instr!(opcodes::F64_CONVERT_I64_U) => self.unop(I32, F64),
            instr!(opcodes::F64_PROMOTE_F32) => self.unop(F32, F64),

            // 0BC
            instr!(opcodes::I32_REINTERPRET_F32) => self.unop(F32, I32),
            instr!(opcodes::I64_REINTERPRET_F64) => self.unop(F64, I64),
            instr!(opcodes::F32_REINTERPRET_I32) => self.unop(I32, F32),
            instr!(opcodes::F64_REINTERPRET_I64) => self.unop(I64, F64),

            // 0xC0
            instr!(opcodes::I32_EXTEND8_S) => self.unop(I32, I32),
            instr!(opcodes::I32_EXTEND16_S) => self.unop(I32, I32),
            instr!(opcodes::I64_EXTEND8_S) => self.unop(I64, I64),
            instr!(opcodes::I64_EXTEND16_S) => self.unop(I64, I64),
            instr!(opcodes::I64_EXTEND32_S) => self.unop(I64, I64),

            instr!(opcodes::REF_NULL => Operands::HeapType(ht)) => {
                let vt = ValueType::Ref(*ht);
                self.stacks.push_val(vt);
                Ok(())
            }
            instr!(opcodes::REF_IS_NULL) => {
                self.stacks.pop_ref()?;
                self.stacks.push_val(I32);
                Ok(())
            }
            instr!(opcodes::REF_FUNC => Operands::FuncIndex(idx)) => {
                ((idx.value() as usize) < self.module.funcs.len())
                    .true_or(ValidationError::UnknownFunc)?;
                self.stacks.push_val(FUNC);
                Ok(())
            }
            // 0xFC 0x00
            instr!(opcodes::I32_TRUNC_SAT_F32_S) => self.unop(F32, I32),
            instr!(opcodes::I32_TRUNC_SAT_F32_U) => self.unop(F32, I32),
            instr!(opcodes::I32_TRUNC_SAT_F64_S) => self.unop(F64, I32),
            instr!(opcodes::I32_TRUNC_SAT_F64_U) => self.unop(F64, I32),
            instr!(opcodes::I64_TRUNC_SAT_F32_S) => self.unop(F32, I64),
            instr!(opcodes::I64_TRUNC_SAT_F32_U) => self.unop(F32, I64),
            instr!(opcodes::I64_TRUNC_SAT_F64_S) => self.unop(F64, I64),
            instr!(opcodes::I64_TRUNC_SAT_F64_U) => self.unop(F64, I64),

            // 0xFC 0x08
            instr!(opcodes::MEMORY_INIT => Operands::DataIndex(idx)) => {
                ((idx.value() as usize) < self.module.datas)
                    .true_or(ValidationError::UnknownData)?;
                (!self.module.mems.is_empty()).true_or(ValidationError::UnknownMemory)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                Ok(())
            }
            instr!(opcodes::DATA_DROP => Operands::DataIndex(idx)) => {
                ((idx.value() as usize) < self.module.datas)
                    .true_or(ValidationError::UnknownData)?;
                Ok(())
            }
            instr!(opcodes::MEMORY_COPY) | instr!(opcodes::MEMORY_FILL) => {
                (0 < self.module.datas).true_or(ValidationError::UnknownMemory)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                Ok(())
            }
            instr!(opcodes::TABLE_INIT => Operands::TableInit(tidx, eidx)) => {
                ((tidx.value() as usize) < self.module.tables.len())
                    .true_or(ValidationError::UnknownTable)?;
                ((eidx.value() as usize) < self.module.elems.len())
                    .true_or(ValidationError::UnknownElem)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                Ok(())
            }
            instr!(opcodes::ELEM_DROP => Operands::ElemIndex(idx)) => {
                ((idx.value() as usize) < self.module.elems.len())
                    .true_or(ValidationError::UnknownElem)?;
                Ok(())
            }
            instr!(opcodes::TABLE_COPY => Operands::TableCopy(srcidx, dstidx)) => {
                ((srcidx.value() as usize) < self.module.tables.len())
                    .true_or(ValidationError::UnknownTable)?;
                ((dstidx.value() as usize) < self.module.tables.len())
                    .true_or(ValidationError::UnknownTable)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                Ok(())
            }
            instr!(opcodes::TABLE_GROW => Operands::TableIndex(idx)) => {
                let tabletype = self
                    .module
                    .tables
                    .get(idx.value() as usize)
                    .ok_or(ValidationError::UnknownTable)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(tabletype.reftype.into())?;
                self.stacks.push_val(I32);
                Ok(())
            }
            instr!(opcodes::TABLE_SIZE => Operands::TableIndex(idx)) => {
                ((idx.value() as usize) < self.module.tables.len())
                    .true_or(ValidationError::UnknownTable)?;
                self.stacks.push_val(I32);
                Ok(())
            }
            instr!(opcodes::TABLE_FILL => Operands::TableIndex(idx)) => {
                let tabletype = self
                    .module
                    .tables
                    .get(idx.value() as usize)
                    .ok_or(ValidationError::UnknownTable)?;
                println!("TABLEF FILL FOR REFTYPE {:?}", tabletype.reftype);
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(tabletype.reftype.into())?;
                self.stacks.pop_val(I32)?;
                Ok(())
            }

            _ => Err(ValidationError::UnhandledInstruction(instr.clone())),
        }
    }
}
