use {
    super::{FunctionType, KindResult as Result, Validation, ValidationErrorKind, ValidationMode},
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
    pub fn validate_instr(&mut self, instr: &Instruction<Resolved>) -> Result<()> {
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
        (!self.module.mems.is_empty()).true_or(ValidationErrorKind::UnknownMemory)?;
        (alignment <= natural_alignment)
            .true_or(ValidationErrorKind::AlignmentTooLarge(alignment))?;
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
        (!self.module.mems.is_empty()).true_or(ValidationErrorKind::UnknownMemory)?;
        (alignment <= natural_alignment)
            .true_or(ValidationErrorKind::AlignmentTooLarge(alignment))?;
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
            .ok_or(ValidationErrorKind::UnknownLocal(idx.clone()))
            .copied()
    }

    pub fn validate_end(&mut self) -> Result<()> {
        self.error_for_mode(|s| {
            let frame = s.stacks.pop_ctrl()?;
            s.stacks.push_vals(&frame.end_types);
            Ok(())
        })
    }

    pub fn validate_else(&mut self) -> Result<()> {
        self.error_for_mode(|s| {
            let frame = s.stacks.pop_ctrl()?;
            (frame.opcode == opcodes::IF).true_or(ValidationErrorKind::OpcodeMismatch)?;
            s.stacks
                .push_ctrl(frame.opcode, frame.start_types, frame.end_types);
            Ok(())
        })
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

            instr!(opcodes::ELSE) => self.validate_else(),

            instr!(opcodes::END) => self.validate_end(),

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
                Ok(())
            }

            instr!(opcodes::BR_TABLE => Operands::BrTable(idxes, last)) => {
                self.stacks.pop_val(I32)?;
                let default_arity = self.stacks.label_arity(last)?;
                for idx in idxes {
                    let break_arity = self.stacks.label_arity(idx)?;
                    (break_arity == default_arity)
                        .true_or(ValidationErrorKind::BreakTypeMismatch)?;
                    let popped = self.stacks.pop_label_types(idx)?;
                    for p in popped.iter() {
                        self.stacks.push_val(*p)
                    }
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
                    .ok_or(ValidationErrorKind::UnknownFunc)?;
                self.stacks.pop_vals(&ft.params)?;
                self.stacks.push_vals(&ft.results);
                Ok(())
            }

            instr!(opcodes::CALL_INDIRECT => Operands::CallIndirect(tabidx, typeuse)) => {
                self.module
                    .tables
                    .get(tabidx.value() as usize)
                    .map(|t| t.reftype == RefType::Func)
                    .ok_or(ValidationErrorKind::UnknownTable)?
                    .true_or(ValidationErrorKind::WrongTableType)?;
                let ft = self
                    .module
                    .types
                    .get(typeuse.index().value() as usize)
                    .ok_or(ValidationErrorKind::UnknownType)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_vals(&ft.params)?;
                self.stacks.push_vals(&ft.results);
                Ok(())
            }

            // Note: the binary 0x1b and 0x1c both turn into SELECT,
            // with opcode 0x1b, but different operand types.
            instr!(opcodes::SELECT) => {
                self.stacks.pop_val(I32)?;
                let v1 = self.stacks.pop_any()?;
                let v2 = self.stacks.pop_any()?;

                match (v1, v2) {
                    (
                        ref v1 @ ValidationType::Value(ValueType::Num(n1)),
                        ref v2 @ ValidationType::Value(ValueType::Num(n2)),
                    ) => {
                        (n1 == n2).true_or(ValidationErrorKind::TypeMismatch {
                            actual: *v1,
                            expect: *v2,
                        })?;
                        self.stacks.push_val(*v1);
                    }
                    (ValidationType::Unknown, v2 @ ValidationType::Value(ValueType::Num(_))) => {
                        self.stacks.push_val(v2);
                    }
                    (v1 @ ValidationType::Value(ValueType::Num(_)), ValidationType::Unknown) => {
                        self.stacks.push_val(v1);
                    }
                    (ValidationType::Unknown, ValidationType::Unknown) => {
                        self.stacks.push_val(ValidationType::Unknown)
                    }
                    (v1, _) => Err(ValidationErrorKind::ExpectedNum { actual: v1 })?,
                };
                Ok(())
            }
            instr!(opcodes::SELECT => Operands::SelectT(t)) => {
                (t.len() == 1).true_or(ValidationErrorKind::UnsupportedSelect)?;
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
            instr!(opcodes::GLOBAL_GET => Operands::GlobalIndex(idx)) => {
                println!("VALIDATING GLOBAL {:?} IN {:?}", idx, self.module.globals);
                let global = self
                    .module
                    .globals
                    .get(idx.value() as usize)
                    .ok_or(ValidationErrorKind::UnknownGlobal)?;
                self.stacks.push_val(global.globaltype.valtype);
                Ok(())
            }
            instr!(opcodes::GLOBAL_SET => Operands::GlobalIndex(idx)) => {
                let global = self
                    .module
                    .globals
                    .get(idx.value() as usize)
                    .ok_or(ValidationErrorKind::UnknownGlobal)?;
                (global.globaltype.mutable).true_or(ValidationErrorKind::ImmutableGlobal)?;
                self.stacks.pop_val(global.globaltype.valtype)?;
                Ok(())
            }
            instr!(opcodes::TABLE_GET => Operands::TableIndex(idx)) => {
                let table = self
                    .module
                    .tables
                    .get(idx.value() as usize)
                    .ok_or(ValidationErrorKind::UnknownTable)?;
                self.stacks.pop_val(I32)?;
                self.stacks.push_val(table.reftype);
                Ok(())
            }
            instr!(opcodes::TABLE_SET => Operands::TableIndex(idx)) => {
                let table = self
                    .module
                    .tables
                    .get(idx.value() as usize)
                    .ok_or(ValidationErrorKind::UnknownTable)?;
                self.stacks.pop_val(table.reftype.into())?;
                self.stacks.pop_val(I32)?;
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
                (!self.module.mems.is_empty()).true_or(ValidationErrorKind::UnknownMemory)?;
                self.stacks.push_val(I32);
                Ok(())
            }
            instr!(opcodes::MEMORY_GROW) => {
                (!self.module.mems.is_empty()).true_or(ValidationErrorKind::UnknownMemory)?;
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
            instr!(opcodes::F32_CONVERT_I64_S) => self.unop(I64, F32),
            instr!(opcodes::F32_CONVERT_I64_U) => self.unop(I64, F32),
            instr!(opcodes::F32_DEMOTE_F64) => self.unop(F64, F32),

            // 0xB7
            instr!(opcodes::F64_CONVERT_I32_S) => self.unop(I32, F64),
            instr!(opcodes::F64_CONVERT_I32_U) => self.unop(I32, F64),
            instr!(opcodes::F64_CONVERT_I64_S) => self.unop(I64, F64),
            instr!(opcodes::F64_CONVERT_I64_U) => self.unop(I64, F64),
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
                match self.stacks.pop_any()? {
                    ValidationType::Value(ValueType::Ref(_)) | ValidationType::Unknown => {}
                    x => Err(ValidationErrorKind::ExpectedRef { actual: x })?,
                };
                self.stacks.push_val(I32);
                Ok(())
            }
            instr!(opcodes::REF_FUNC => Operands::FuncIndex(idx)) => {
                ((idx.value() as usize) < self.module.funcs.len())
                    .true_or(ValidationErrorKind::UnknownFunc)?;
                (self.module.funcrefs.contains(idx))
                    .true_or(ValidationErrorKind::UndeclaredFunctionRef)?;
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
                (!self.module.mems.is_empty()).true_or(ValidationErrorKind::UnknownMemory)?;
                ((idx.value() as usize) < self.module.datas)
                    .true_or(ValidationErrorKind::UnknownData)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                Ok(())
            }
            instr!(opcodes::DATA_DROP => Operands::DataIndex(idx)) => {
                ((idx.value() as usize) < self.module.datas)
                    .true_or(ValidationErrorKind::UnknownData)?;
                Ok(())
            }
            instr!(opcodes::MEMORY_COPY) | instr!(opcodes::MEMORY_FILL) => {
                (!self.module.mems.is_empty()).true_or(ValidationErrorKind::UnknownMemory)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                Ok(())
            }
            instr!(opcodes::TABLE_INIT => Operands::TableInit(tidx, eidx)) => {
                let table = self
                    .module
                    .tables
                    .get(tidx.value() as usize)
                    .ok_or(ValidationErrorKind::UnknownTable)?;
                let elemtype = self
                    .module
                    .elems
                    .get(eidx.value() as usize)
                    .ok_or(ValidationErrorKind::UnknownElem)?;
                (&table.reftype == elemtype).true_or(ValidationErrorKind::TypeMismatch {
                    actual: (*elemtype).into(),
                    expect: table.reftype.into(),
                })?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(I32)?;
                Ok(())
            }
            instr!(opcodes::ELEM_DROP => Operands::ElemIndex(idx)) => {
                ((idx.value() as usize) < self.module.elems.len())
                    .true_or(ValidationErrorKind::UnknownElem)?;
                Ok(())
            }
            instr!(opcodes::TABLE_COPY => Operands::TableCopy(srcidx, dstidx)) => {
                let t1 = self
                    .module
                    .tables
                    .get(srcidx.value() as usize)
                    .ok_or(ValidationErrorKind::UnknownTable)?;
                let t2 = self
                    .module
                    .tables
                    .get(dstidx.value() as usize)
                    .ok_or(ValidationErrorKind::UnknownTable)?;
                (t1.reftype == t2.reftype).true_or(ValidationErrorKind::TypeMismatch {
                    actual: t2.reftype.into(),
                    expect: t1.reftype.into(),
                })?;
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
                    .ok_or(ValidationErrorKind::UnknownTable)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(tabletype.reftype.into())?;
                self.stacks.push_val(I32);
                Ok(())
            }
            instr!(opcodes::TABLE_SIZE => Operands::TableIndex(idx)) => {
                ((idx.value() as usize) < self.module.tables.len())
                    .true_or(ValidationErrorKind::UnknownTable)?;
                self.stacks.push_val(I32);
                Ok(())
            }
            instr!(opcodes::TABLE_FILL => Operands::TableIndex(idx)) => {
                let tabletype = self
                    .module
                    .tables
                    .get(idx.value() as usize)
                    .ok_or(ValidationErrorKind::UnknownTable)?;
                self.stacks.pop_val(I32)?;
                self.stacks.pop_val(tabletype.reftype.into())?;
                self.stacks.pop_val(I32)?;
                Ok(())
            }

            _ => Err(ValidationErrorKind::UnhandledInstruction(instr.clone())),
        }
    }
}
