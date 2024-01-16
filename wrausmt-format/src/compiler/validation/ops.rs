use {
    super::{Result, Validation, ValidationError, ValidationMode},
    wrausmt_runtime::{
        instructions::opcodes,
        syntax::{
            types::{NumType, ValueType},
            Instruction, Operands, Resolved,
        },
    },
};

// Conveniences for implementing the checks below.
const I32: ValueType = ValueType::Num(NumType::I32);
const I64: ValueType = ValueType::Num(NumType::I64);
const F32: ValueType = ValueType::Num(NumType::F32);
const F64: ValueType = ValueType::Num(NumType::F64);

impl<'a> Validation<'a> {
    /// Validate one instruction. The returned error will respect the
    /// [`ValidationMode`] provided at creation.
    pub fn handle_instr(&mut self, instr: &Instruction<Resolved>) -> Result<()> {
        self.error_for_mode(|s| s.validation_result(instr))
    }

    /// Finalize usage of this [`Validation`]. The final result stack is
    /// verified. The returned error will response the [`ValidationMode`]
    /// provided at creation.
    pub fn finish(mut self) -> Result<()> {
        self.error_for_mode(Self::validate_results)
    }

    fn validate_results(&mut self) -> Result<()> {
        for r in self.resulttypes {
            self.pop_expect(*r)?;
        }
        Ok(())
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

    fn noargs(&mut self, o: ValueType) -> Result<()> {
        self.push_val(o);
        Ok(())
    }

    fn unop(&mut self, i: ValueType, o: ValueType) -> Result<()> {
        self.pop_expect(i)?;
        self.push_val(o);
        Ok(())
    }

    fn binop(&mut self, a: ValueType, b: ValueType, o: ValueType) -> Result<()> {
        self.pop_expect(a)?;
        self.pop_expect(b)?;
        self.push_val(o);
        Ok(())
    }

    fn local_type(&mut self, operands: &Operands<Resolved>) -> Result<ValueType> {
        match operands {
            Operands::LocalIndex(li) => Ok(*self
                .localtypes
                .get(li.value() as usize)
                .ok_or(ValidationError::UnknownLocal(li.clone()))?),
            _ => panic!("Wrong operands for local, impl error."),
        }
    }

    fn validation_result(&mut self, instr: &Instruction<Resolved>) -> Result<()> {
        match instr.opcode {
            opcodes::UNREACHABLE => self.unreachable(),

            opcodes::END => {
                // TODO
                Ok(())
            }
            // 0x20
            opcodes::LOCAL_GET => {
                let ty = self.local_type(&instr.operands)?;
                self.push_val(ty);
                Ok(())
            }
            opcodes::LOCAL_SET => {
                let ty = self.local_type(&instr.operands)?;
                self.pop_expect(ty)?;
                Ok(())
            }
            opcodes::LOCAL_TEE => {
                let ty = self.local_type(&instr.operands)?;
                self.pop_expect(ty)?;
                self.push_val(ty);
                self.push_val(ty);
                Ok(())
            }
            // 0x41
            opcodes::I32_CONST => self.noargs(I32),
            opcodes::I64_CONST => self.noargs(I64),
            opcodes::F32_CONST => self.noargs(F32),
            opcodes::F64_CONST => self.noargs(F64),

            // 0x45
            opcodes::I32_EQZ => self.unop(I32, I32),
            opcodes::I32_EQ => self.binop(I32, I32, I32),
            opcodes::I32_NE => self.binop(I32, I32, I32),
            opcodes::I32_LT_S => self.binop(I32, I32, I32),
            opcodes::I32_LT_U => self.binop(I32, I32, I32),
            opcodes::I32_GT_S => self.binop(I32, I32, I32),
            opcodes::I32_GT_U => self.binop(I32, I32, I32),
            opcodes::I32_LE_S => self.binop(I32, I32, I32),
            opcodes::I32_LE_U => self.binop(I32, I32, I32),
            opcodes::I32_GE_S => self.binop(I32, I32, I32),
            opcodes::I32_GE_U => self.binop(I32, I32, I32),

            // 0x50
            opcodes::I64_EQZ => self.unop(I64, I32),
            opcodes::I64_EQ => self.binop(I64, I64, I32),
            opcodes::I64_NE => self.binop(I64, I64, I32),
            opcodes::I64_LT_S => self.binop(I64, I64, I32),
            opcodes::I64_LT_U => self.binop(I64, I64, I32),
            opcodes::I64_GT_S => self.binop(I64, I64, I32),
            opcodes::I64_GT_U => self.binop(I64, I64, I32),
            opcodes::I64_LE_S => self.binop(I64, I64, I32),
            opcodes::I64_LE_U => self.binop(I64, I64, I32),
            opcodes::I64_GE_S => self.binop(I64, I64, I32),
            opcodes::I64_GE_U => self.binop(I64, I64, I32),

            // 0x5B
            opcodes::F32_EQ => self.binop(F32, F32, I32),
            opcodes::F32_NE => self.binop(F32, F32, I32),
            opcodes::F32_LT => self.binop(F32, F32, I32),
            opcodes::F32_GT => self.binop(F32, F32, I32),
            opcodes::F32_LE => self.binop(F32, F32, I32),
            opcodes::F32_GE => self.binop(F32, F32, I32),

            // 0x61
            opcodes::F64_EQ => self.binop(F64, F64, I32),
            opcodes::F64_NE => self.binop(F64, F64, I32),
            opcodes::F64_LT => self.binop(F64, F64, I32),
            opcodes::F64_GT => self.binop(F64, F64, I32),
            opcodes::F64_LE => self.binop(F64, F64, I32),
            opcodes::F64_GE => self.binop(F64, F64, I32),

            // 0x67
            opcodes::I32_CLZ => self.unop(I32, I32),
            opcodes::I32_CTZ => self.unop(I32, I32),
            opcodes::I32_POPCNT => self.unop(I32, I32),
            opcodes::I32_ADD => self.binop(I32, I32, I32),
            opcodes::I32_SUB => self.binop(I32, I32, I32),
            opcodes::I32_MUL => self.binop(I32, I32, I32),
            opcodes::I32_DIV_S => self.binop(I32, I32, I32),
            opcodes::I32_DIV_U => self.binop(I32, I32, I32),
            opcodes::I32_REM_S => self.binop(I32, I32, I32),
            opcodes::I32_REM_U => self.binop(I32, I32, I32),
            opcodes::I32_AND => self.binop(I32, I32, I32),
            opcodes::I32_OR => self.binop(I32, I32, I32),
            opcodes::I32_XOR => self.binop(I32, I32, I32),
            opcodes::I32_SHL => self.binop(I32, I32, I32),
            opcodes::I32_SHR_S => self.binop(I32, I32, I32),
            opcodes::I32_SHR_U => self.binop(I32, I32, I32),
            opcodes::I32_ROTL => self.binop(I32, I32, I32),
            opcodes::I32_ROTR => self.binop(I32, I32, I32),

            // 0x79
            opcodes::I64_CLZ => self.unop(I64, I64),
            opcodes::I64_CTZ => self.unop(I64, I64),
            opcodes::I64_POPCNT => self.unop(I64, I64),
            opcodes::I64_ADD => self.binop(I64, I64, I64),
            opcodes::I64_SUB => self.binop(I64, I64, I64),
            opcodes::I64_MUL => self.binop(I64, I64, I64),
            opcodes::I64_DIV_S => self.binop(I64, I64, I64),
            opcodes::I64_DIV_U => self.binop(I64, I64, I64),
            opcodes::I64_REM_S => self.binop(I64, I64, I64),
            opcodes::I64_REM_U => self.binop(I64, I64, I64),
            opcodes::I64_AND => self.binop(I64, I64, I64),
            opcodes::I64_OR => self.binop(I64, I64, I64),
            opcodes::I64_XOR => self.binop(I64, I64, I64),
            opcodes::I64_SHL => self.binop(I64, I64, I64),
            opcodes::I64_SHR_S => self.binop(I64, I64, I64),
            opcodes::I64_SHR_U => self.binop(I64, I64, I64),
            opcodes::I64_ROTL => self.binop(I64, I64, I64),
            opcodes::I64_ROTR => self.binop(I64, I64, I64),

            // 0x8B
            opcodes::F32_ABS => self.unop(F32, F32),
            opcodes::F32_NEG => self.unop(F32, F32),
            opcodes::F32_CEIL => self.unop(F32, F32),
            opcodes::F32_FLOOR => self.unop(F32, F32),
            opcodes::F32_TRUNC => self.unop(F32, F32),
            opcodes::F32_NEAREST => self.unop(F32, F32),
            opcodes::F32_SQRT => self.unop(F32, F32),

            // 0x92
            opcodes::F32_ADD => self.binop(F32, F32, F32),
            opcodes::F32_SUB => self.binop(F32, F32, F32),
            opcodes::F32_MUL => self.binop(F32, F32, F32),
            opcodes::F32_DIV => self.binop(F32, F32, F32),
            opcodes::F32_MIN => self.binop(F32, F32, F32),
            opcodes::F32_MAX => self.binop(F32, F32, F32),
            opcodes::F32_COPYSIGN => self.binop(F32, F32, F32),

            // 0x99
            opcodes::F64_ABS => self.unop(F64, F64),
            opcodes::F64_NEG => self.unop(F64, F64),
            opcodes::F64_CEIL => self.unop(F64, F64),
            opcodes::F64_FLOOR => self.unop(F64, F64),
            opcodes::F64_TRUNC => self.unop(F64, F64),
            opcodes::F64_NEAREST => self.unop(F64, F64),
            opcodes::F64_SQRT => self.unop(F64, F64),

            // 0xA0
            opcodes::F64_ADD => self.binop(F64, F64, F64),
            opcodes::F64_SUB => self.binop(F64, F64, F64),
            opcodes::F64_MUL => self.binop(F64, F64, F64),
            opcodes::F64_DIV => self.binop(F64, F64, F64),
            opcodes::F64_MIN => self.binop(F64, F64, F64),
            opcodes::F64_MAX => self.binop(F64, F64, F64),
            opcodes::F64_COPYSIGN => self.binop(F64, F64, F64),

            // 0xA7
            opcodes::I32_WRAP_I64 => self.unop(I64, I32),
            opcodes::I32_TRUNC_F32_S => self.unop(F32, I32),
            opcodes::I32_TRUNC_F32_U => self.unop(F32, I32),
            opcodes::I32_TRUNC_F64_S => self.unop(F64, I32),
            opcodes::I32_TRUNC_F64_U => self.unop(F64, I32),

            // 0xAC
            opcodes::I64_EXTEND_I32_S => self.unop(I32, I64),
            opcodes::I64_EXTEND_I32_U => self.unop(I32, I64),
            opcodes::I64_TRUNC_F32_S => self.unop(F32, I64),
            opcodes::I64_TRUNC_F32_U => self.unop(F32, I64),
            opcodes::I64_TRUNC_F64_S => self.unop(F64, I64),
            opcodes::I64_TRUNC_F64_U => self.unop(F64, I64),

            // 0xB2
            opcodes::F32_CONVERT_I32_S => self.unop(I32, F32),
            opcodes::F32_CONVERT_I32_U => self.unop(I32, F32),
            opcodes::F32_CONVERT_I64_S => self.unop(I32, F32),
            opcodes::F32_CONVERT_I64_U => self.unop(I32, F32),
            opcodes::F32_DEMOTE_F64 => self.unop(F64, F32),

            // 0xB7
            opcodes::F64_CONVERT_I32_S => self.unop(I32, F64),
            opcodes::F64_CONVERT_I32_U => self.unop(I32, F64),
            opcodes::F64_CONVERT_I64_S => self.unop(I32, F64),
            opcodes::F64_CONVERT_I64_U => self.unop(I32, F64),
            opcodes::F64_PROMOTE_F32 => self.unop(F32, F64),

            // 0BC
            opcodes::I32_REINTERPRET_F32 => self.unop(F32, I32),
            opcodes::I64_REINTERPRET_F64 => self.unop(F64, I64),
            opcodes::F32_REINTERPRET_I32 => self.unop(I32, F32),
            opcodes::F64_REINTERPRET_I64 => self.unop(I64, F64),

            // 0xC0
            opcodes::I32_EXTEND8_S => self.unop(I32, I32),
            opcodes::I32_EXTEND16_S => self.unop(I32, I32),
            opcodes::I64_EXTEND8_S => self.unop(I64, I64),
            opcodes::I64_EXTEND16_S => self.unop(I64, I64),
            opcodes::I64_EXTEND32_S => self.unop(I64, I64),

            // 0xFC 0x00
            opcodes::I32_TRUNC_SAT_F32_S => self.unop(F32, I32),
            opcodes::I32_TRUNC_SAT_F32_U => self.unop(F32, I32),
            opcodes::I32_TRUNC_SAT_F64_S => self.unop(F64, I32),
            opcodes::I32_TRUNC_SAT_F64_U => self.unop(F64, I32),
            opcodes::I64_TRUNC_SAT_F32_S => self.unop(F32, I64),
            opcodes::I64_TRUNC_SAT_F32_U => self.unop(F32, I64),
            opcodes::I64_TRUNC_SAT_F64_S => self.unop(F64, I64),
            opcodes::I64_TRUNC_SAT_F64_U => self.unop(F64, I64),
            _ => Err(ValidationError::UnknownOpcode(instr.opcode)),
        }
    }
}
