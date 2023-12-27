/// This file was generated automatically by the codegen crate.
/// Do not edit it manually.
///
/// See build.rs for wrausmt or the included codegen crate for more details.
use crate::runtime::error::Result;
use crate::runtime::{
    error::TrapKind,
    exec::{ExecutionContext, ExecutionContextActions},
    values::Ref,
};

#[allow(dead_code)]
pub fn unreachable_exec(_ec: &mut ExecutionContext) -> Result<()> {
    Err(TrapKind::Unreachable.into())
}

#[allow(dead_code)]
pub fn nop_exec(_ec: &mut ExecutionContext) -> Result<()> {
    Ok(())
}

#[allow(dead_code)]
pub fn block_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.push_label_end()
}

#[allow(dead_code)]
pub fn loop_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.push_label_start()
}

#[allow(dead_code)]
pub fn if_exec(_ec: &mut ExecutionContext) -> Result<()> {
    // Note: pop the condition first,
    // so that push_label calculates the right stack
    // return location.
    let cnd = _ec.pop::<i32>()?;
    _ec.push_label_end()?;
    let el = _ec.op_u32()?;
    if cnd == 0 {
        _ec.continuation(el)?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn else_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.br(0)
}

#[allow(dead_code)]
pub fn end_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.pop_label()?;
    Ok(())
}

#[allow(dead_code)]
pub fn br_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let labelidx = _ec.op_u32()?;
    _ec.br(labelidx)
}

#[allow(dead_code)]
pub fn br_if_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let l = _ec.op_u32()?;
    let c = _ec.pop::<u32>()?;
    if c != 0 {
        _ec.br(l)?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn br_table_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let icnt = _ec.op_u32()?;
    let mut indices: Vec<u32> = Vec::with_capacity(icnt as usize);
    for _ in 0..icnt {
        indices.push(_ec.op_u32()?)
    }
    let sel = std::cmp::min(icnt - 1, _ec.pop::<u32>()?);
    _ec.br(indices[sel as usize])
}

#[allow(dead_code)]
pub fn return_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.ret()
}

#[allow(dead_code)]
pub fn call_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let i = _ec.op_u32()?;
    _ec.call(i)
}

#[allow(dead_code)]
pub fn call_indirect_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let t = _ec.op_u32()?;
    let tu = _ec.op_u32()?;
    let ei = _ec.pop::<u32>()?;
    let f = _ec.get_func_table(t, ei)?;
    _ec.call_addr(f, tu)
}

#[allow(dead_code)]
pub fn drop_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.pop_value()?;
    Ok(())
}

#[allow(dead_code)]
pub fn select_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let s = _ec.pop::<u32>()?;
    let v2 = _ec.pop_value()?;
    let v1 = _ec.pop_value()?;
    if s == 0 {
        _ec.push(v2)
    } else {
        _ec.push(v1)
    }
}

#[allow(dead_code)]
pub fn local_get_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let i = _ec.op_u32()?;
    let v = _ec.get_local(i)?;
    _ec.push_value(v)
}

#[allow(dead_code)]
pub fn local_set_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let i = _ec.op_u32()?;
    let v = _ec.pop_value()?;
    _ec.set_local(i, v)
}

#[allow(dead_code)]
pub fn local_tee_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let i = _ec.op_u32()?;
    let v = _ec.pop_value()?;
    _ec.push_value(v)?;
    _ec.set_local(i, v)
}

#[allow(dead_code)]
pub fn global_get_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let i = _ec.op_u32()?;
    let v = _ec.get_global(i)?;
    _ec.push_value(v)
}

#[allow(dead_code)]
pub fn global_set_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let i = _ec.op_u32()?;
    let v = _ec.pop_value()?;
    _ec.set_global(i, v)
}

#[allow(dead_code)]
pub fn table_get_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let t = _ec.op_u32()?;
    let i = _ec.pop::<u32>()?;
    let v = _ec.get_table_elem(t, i)?;
    _ec.push_value(v.into())
}

#[allow(dead_code)]
pub fn table_set_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let t = _ec.op_u32()?;
    let v = _ec.pop_value()?;
    let i = _ec.pop::<u32>()?;
    _ec.set_table_elem(t, i, v)
}

#[allow(dead_code)]
pub fn i32_load_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<4>()?;
    let v = u32::from_le_bytes(bs);
    _ec.push(v)
}

#[allow(dead_code)]
pub fn i64_load_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<8>()?;
    let v = u64::from_le_bytes(bs);
    _ec.push(v)
}

#[allow(dead_code)]
pub fn f32_load_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<4>()?;
    let v = f32::from_le_bytes(bs);
    _ec.push(v)
}

#[allow(dead_code)]
pub fn f64_load_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<8>()?;
    let v = f64::from_le_bytes(bs);
    _ec.push(v)
}

#[allow(dead_code)]
pub fn i32_load8_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<1>()?;
    let v = i8::from_le_bytes(bs);
    _ec.push(v as i32)
}

#[allow(dead_code)]
pub fn i32_load8_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<1>()?;
    let v = u8::from_le_bytes(bs);
    _ec.push(v as u32)
}

#[allow(dead_code)]
pub fn i32_load16_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<2>()?;
    let v = i16::from_le_bytes(bs);
    _ec.push(v as i32)
}

#[allow(dead_code)]
pub fn i32_load16_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<2>()?;
    let v = u16::from_le_bytes(bs);
    _ec.push(v as u32)
}

#[allow(dead_code)]
pub fn i64_load8_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<1>()?;
    let v = i8::from_le_bytes(bs);
    _ec.push(v as i64)
}

#[allow(dead_code)]
pub fn i64_load8_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<1>()?;
    let v = u8::from_le_bytes(bs);
    _ec.push(v as i64)
}

#[allow(dead_code)]
pub fn i64_load16_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<2>()?;
    let v = i16::from_le_bytes(bs);
    _ec.push(v as i64)
}

#[allow(dead_code)]
pub fn i64_load16_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<2>()?;
    let v = u16::from_le_bytes(bs);
    _ec.push(v as u64)
}

#[allow(dead_code)]
pub fn i64_load32_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<4>()?;
    let v = i32::from_le_bytes(bs);
    _ec.push(v as i64)
}

#[allow(dead_code)]
pub fn i64_load32_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let bs = _ec.get_mem::<4>()?;
    let v = u32::from_le_bytes(bs);
    _ec.push(v as u64)
}

#[allow(dead_code)]
pub fn i32_store_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.pop::<u32>()?;
    let bs = v.to_le_bytes();
    _ec.put_mem::<4>(bs)
}

#[allow(dead_code)]
pub fn i64_store_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.pop::<u64>()?;
    let bs = v.to_le_bytes();
    _ec.put_mem::<8>(bs)
}

#[allow(dead_code)]
pub fn f32_store_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.pop::<f32>()?;
    let bs = v.to_le_bytes();
    _ec.put_mem::<4>(bs)
}

#[allow(dead_code)]
pub fn f64_store_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.pop::<f64>()?;
    let bs = v.to_le_bytes();
    _ec.put_mem::<8>(bs)
}

#[allow(dead_code)]
pub fn i32_store8_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.pop::<u32>()? as u8;
    let bs = v.to_le_bytes();
    _ec.put_mem::<1>(bs)
}

#[allow(dead_code)]
pub fn i32_store16_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.pop::<u32>()? as u16;
    let bs = v.to_le_bytes();
    _ec.put_mem::<2>(bs)
}

#[allow(dead_code)]
pub fn i64_store8_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.pop::<u64>()? as u8;
    let bs = v.to_le_bytes();
    _ec.put_mem::<1>(bs)
}

#[allow(dead_code)]
pub fn i64_store16_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.pop::<u64>()? as u16;
    let bs = v.to_le_bytes();
    _ec.put_mem::<2>(bs)
}

#[allow(dead_code)]
pub fn i64_store32_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.pop::<u64>()? as u32;
    let bs = v.to_le_bytes();
    _ec.put_mem::<4>(bs)
}

#[allow(dead_code)]
pub fn memory_size_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.mem_size()
}

#[allow(dead_code)]
pub fn memory_grow_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.mem_grow()
}

#[allow(dead_code)]
pub fn i32_const_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.op_u32()?;
    _ec.push_value(v.into())
}

#[allow(dead_code)]
pub fn i64_const_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.op_u64()?;
    _ec.push_value(v.into())
}

#[allow(dead_code)]
pub fn f32_const_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.op_u32()?;
    let d = f32::from_bits(v);
    _ec.push_value(d.into())
}

#[allow(dead_code)]
pub fn f64_const_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let v = _ec.op_u64()?;
    let d = f64::from_bits(v);
    _ec.push_value(d.into())
}

#[allow(dead_code)]
pub fn i32_eqz_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.testop::<u32>(|i| i == 0)
}

#[allow(dead_code)]
pub fn i32_eq_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<u32>(|l, r| l == r)
}

#[allow(dead_code)]
pub fn i32_ne_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<u32>(|l, r| l != r)
}

#[allow(dead_code)]
pub fn i32_lt_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<i32>(|l, r| l < r)
}

#[allow(dead_code)]
pub fn i32_lt_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<u32>(|l, r| l < r)
}

#[allow(dead_code)]
pub fn i32_gt_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<i32>(|l, r| l > r)
}

#[allow(dead_code)]
pub fn i32_gt_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<u32>(|l, r| l > r)
}

#[allow(dead_code)]
pub fn i32_le_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<i32>(|l, r| l <= r)
}

#[allow(dead_code)]
pub fn i32_le_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<u32>(|l, r| l <= r)
}

#[allow(dead_code)]
pub fn i32_ge_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<i32>(|l, r| l >= r)
}

#[allow(dead_code)]
pub fn i32_ge_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<u32>(|l, r| l >= r)
}

#[allow(dead_code)]
pub fn i64_eqz_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.testop::<u64>(|i| i == 0)
}

#[allow(dead_code)]
pub fn i64_eq_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<u64>(|l, r| l == r)
}

#[allow(dead_code)]
pub fn i64_ne_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<u64>(|l, r| l != r)
}

#[allow(dead_code)]
pub fn i64_lt_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<i64>(|l, r| l < r)
}

#[allow(dead_code)]
pub fn i64_lt_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<u64>(|l, r| l < r)
}

#[allow(dead_code)]
pub fn i64_gt_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<i64>(|l, r| l > r)
}

#[allow(dead_code)]
pub fn i64_gt_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<u64>(|l, r| l > r)
}

#[allow(dead_code)]
pub fn i64_le_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<i64>(|l, r| l <= r)
}

#[allow(dead_code)]
pub fn i64_le_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<u64>(|l, r| l <= r)
}

#[allow(dead_code)]
pub fn i64_ge_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<i64>(|l, r| l >= r)
}

#[allow(dead_code)]
pub fn i64_ge_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<u64>(|l, r| l >= r)
}

#[allow(dead_code)]
pub fn f32_eq_exec(_ec: &mut ExecutionContext) -> Result<()> {
    #[allow(clippy::float_cmp)]
    _ec.relop::<f32>(|l, r| l == r)
}

#[allow(dead_code)]
pub fn f32_ne_exec(_ec: &mut ExecutionContext) -> Result<()> {
    #[allow(clippy::float_cmp)]
    _ec.relop::<f32>(|l, r| l != r)
}

#[allow(dead_code)]
pub fn f32_lt_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<f32>(|l, r| l < r)
}

#[allow(dead_code)]
pub fn f32_gt_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<f32>(|l, r| l > r)
}

#[allow(dead_code)]
pub fn f32_le_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<f32>(|l, r| l <= r)
}

#[allow(dead_code)]
pub fn f32_ge_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<f32>(|l, r| l >= r)
}

#[allow(dead_code)]
pub fn f64_eq_exec(_ec: &mut ExecutionContext) -> Result<()> {
    #[allow(clippy::float_cmp)]
    _ec.relop::<f64>(|l, r| l == r)
}

#[allow(dead_code)]
pub fn f64_ne_exec(_ec: &mut ExecutionContext) -> Result<()> {
    #[allow(clippy::float_cmp)]
    _ec.relop::<f64>(|l, r| l != r)
}

#[allow(dead_code)]
pub fn f64_lt_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<f64>(|l, r| l < r)
}

#[allow(dead_code)]
pub fn f64_gt_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<f64>(|l, r| l > r)
}

#[allow(dead_code)]
pub fn f64_le_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<f64>(|l, r| l <= r)
}

#[allow(dead_code)]
pub fn f64_ge_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.relop::<f64>(|l, r| l >= r)
}

#[allow(dead_code)]
pub fn i32_clz_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<u32>(|o| o.leading_zeros())
}

#[allow(dead_code)]
pub fn i32_ctz_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<u32>(|o| o.trailing_zeros())
}

#[allow(dead_code)]
pub fn i32_popcnt_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<u32>(|o| o.count_ones())
}

#[allow(dead_code)]
pub fn i32_add_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u32>(|l, r| l.wrapping_add(r))
}

#[allow(dead_code)]
pub fn i32_sub_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u32>(|l, r| l.wrapping_sub(r))
}

#[allow(dead_code)]
pub fn i32_mul_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u32>(|l, r| l.wrapping_mul(r))
}

#[allow(dead_code)]
pub fn i32_div_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop_trap::<i32>(|l, r| {
        if r == 0 {
            return Err(TrapKind::IntegerDivideByZero);
        }
        if l == i32::MIN && r == -1 {
            return Err(TrapKind::IntegerOverflow);
        }
        Ok(l.wrapping_div(r))
    })
}

#[allow(dead_code)]
pub fn i32_div_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop_trap::<u32>(|l, r| {
        if r == 0 {
            return Err(TrapKind::IntegerDivideByZero);
        }
        Ok(l.wrapping_div(r))
    })
}

#[allow(dead_code)]
pub fn i32_rem_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop_trap::<i32>(|l, r| {
        if r == 0 {
            return Err(TrapKind::IntegerDivideByZero);
        }
        Ok(l.wrapping_rem(r))
    })
}

#[allow(dead_code)]
pub fn i32_rem_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop_trap::<u32>(|l, r| {
        if r == 0 {
            return Err(TrapKind::IntegerDivideByZero);
        }
        Ok(l.wrapping_rem(r))
    })
}

#[allow(dead_code)]
pub fn i32_and_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u32>(|l, r| l & r)
}

#[allow(dead_code)]
pub fn i32_or_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u32>(|l, r| l | r)
}

#[allow(dead_code)]
pub fn i32_xor_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u32>(|l, r| l ^ r)
}

#[allow(dead_code)]
pub fn i32_shl_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u32>(|l, r| l.wrapping_shl(r))
}

#[allow(dead_code)]
pub fn i32_shr_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<i32>(|l, r| l.wrapping_shr(r as u32))
}

#[allow(dead_code)]
pub fn i32_shr_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u32>(|l, r| l.wrapping_shr(r))
}

#[allow(dead_code)]
pub fn i32_rotl_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u32>(|l, r| l.rotate_left(r))
}

#[allow(dead_code)]
pub fn i32_rotr_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u32>(|l, r| l.rotate_right(r))
}

#[allow(dead_code)]
pub fn i64_clz_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<u64>(|o| o.leading_zeros() as u64)
}

#[allow(dead_code)]
pub fn i64_ctz_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<u64>(|o| o.trailing_zeros() as u64)
}

#[allow(dead_code)]
pub fn i64_popcnt_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<u64>(|o| o.count_ones() as u64)
}

#[allow(dead_code)]
pub fn i64_add_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u64>(|l, r| l.wrapping_add(r))
}

#[allow(dead_code)]
pub fn i64_sub_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u64>(|l, r| l.wrapping_sub(r))
}

#[allow(dead_code)]
pub fn i64_mul_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u64>(|l, r| l.wrapping_mul(r))
}

#[allow(dead_code)]
pub fn i64_div_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop_trap::<i64>(|l, r| {
        if r == 0 {
            return Err(TrapKind::IntegerDivideByZero);
        }
        if l == i64::MIN && r == -1 {
            return Err(TrapKind::IntegerOverflow);
        }
        Ok(l.wrapping_div(r))
    })
}

#[allow(dead_code)]
pub fn i64_div_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop_trap::<u64>(|l, r| {
        if r == 0 {
            return Err(TrapKind::IntegerDivideByZero);
        }
        Ok(l.wrapping_div(r))
    })
}

#[allow(dead_code)]
pub fn i64_rem_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop_trap::<i64>(|l, r| {
        if r == 0 {
            return Err(TrapKind::IntegerDivideByZero);
        }
        Ok(l.wrapping_rem(r))
    })
}

#[allow(dead_code)]
pub fn i64_rem_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop_trap::<u64>(|l, r| {
        if r == 0 {
            return Err(TrapKind::IntegerDivideByZero);
        }
        Ok(l.wrapping_rem(r))
    })
}

#[allow(dead_code)]
pub fn i64_and_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u64>(|l, r| l & r)
}

#[allow(dead_code)]
pub fn i64_or_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u64>(|l, r| l | r)
}

#[allow(dead_code)]
pub fn i64_xor_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u64>(|l, r| l ^ r)
}

#[allow(dead_code)]
pub fn i64_shl_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u64>(|l, r| l.wrapping_shl(r as u32))
}

#[allow(dead_code)]
pub fn i64_shr_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<i64>(|l, r| l.wrapping_shr(r as u32))
}

#[allow(dead_code)]
pub fn i64_shr_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u64>(|l, r| l.wrapping_shr(r as u32))
}

#[allow(dead_code)]
pub fn i64_rotl_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u64>(|l, r| l.rotate_left(r as u32))
}

#[allow(dead_code)]
pub fn i64_rotr_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<u64>(|l, r| l.rotate_right(r as u32))
}

#[allow(dead_code)]
pub fn f32_abs_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f32>(|i| i.abs())
}

#[allow(dead_code)]
pub fn f32_neg_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f32>(|i| -i)
}

#[allow(dead_code)]
pub fn f32_ceil_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f32>(|i| i.ceil())
}

#[allow(dead_code)]
pub fn f32_floor_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f32>(|i| i.floor())
}

#[allow(dead_code)]
pub fn f32_trunc_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f32>(|i| i.trunc())
}

#[allow(dead_code)]
pub fn f32_nearest_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f32>(|i| {
        // Round-to-nearest, ties to even
        let tr = i.trunc();
        if tr.to_bits() == i.to_bits() {
            return i;
        }
        tr + (tr % 2.0).copysign(i)
    })
}

#[allow(dead_code)]
pub fn f32_sqrt_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f32>(|i| i.sqrt())
}

#[allow(dead_code)]
pub fn f32_add_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f32>(|l, r| l + r)
}

#[allow(dead_code)]
pub fn f32_sub_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f32>(|l, r| l - r)
}

#[allow(dead_code)]
pub fn f32_mul_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f32>(|l, r| l * r)
}

#[allow(dead_code)]
pub fn f32_div_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f32>(|l, r| l / r)
}

#[allow(dead_code)]
pub fn f32_min_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f32>(|l, r| {
        if l.is_nan() {
            return l;
        }
        if r.is_nan() {
            return r;
        }
        if l == f32::NEG_INFINITY || r == f32::NEG_INFINITY {
            return f32::NEG_INFINITY;
        }
        if l == f32::INFINITY {
            return r;
        }
        if r == f32::INFINITY {
            return l;
        }
        if l == 0.0 && r == 0.0 {
            if l.is_sign_negative() {
                return l;
            }
            return r;
        }
        l.min(r)
    })
}

#[allow(dead_code)]
pub fn f32_max_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f32>(|l, r| {
        if l.is_nan() {
            return l;
        }
        if r.is_nan() {
            return r;
        }
        if l == f32::INFINITY || r == f32::INFINITY {
            return f32::INFINITY;
        }
        if l == f32::NEG_INFINITY {
            return r;
        }
        if r == f32::NEG_INFINITY {
            return l;
        }
        if l == 0.0 && r == 0.0 {
            if l.is_sign_positive() {
                return l;
            }
            return r;
        }
        l.max(r)
    })
}

#[allow(dead_code)]
pub fn f32_copysign_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f32>(|l, r| l.copysign(r))
}

#[allow(dead_code)]
pub fn f64_abs_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f64>(|i| i.abs())
}

#[allow(dead_code)]
pub fn f64_neg_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f64>(|i| -i)
}

#[allow(dead_code)]
pub fn f64_ceil_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f64>(|i| i.ceil())
}

#[allow(dead_code)]
pub fn f64_floor_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f64>(|i| i.floor())
}

#[allow(dead_code)]
pub fn f64_trunc_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f64>(|i| i.trunc())
}

#[allow(dead_code)]
pub fn f64_nearest_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f64>(|i| {
        // Round-to-nearest, ties to even
        let tr = i.trunc();
        if tr.to_bits() == i.to_bits() {
            return i;
        }
        tr + (tr % 2.0).copysign(i)
    })
}

#[allow(dead_code)]
pub fn f64_sqrt_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<f64>(|i| i.sqrt())
}

#[allow(dead_code)]
pub fn f64_add_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f64>(|l, r| l + r)
}

#[allow(dead_code)]
pub fn f64_sub_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f64>(|l, r| l - r)
}

#[allow(dead_code)]
pub fn f64_mul_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f64>(|l, r| l * r)
}

#[allow(dead_code)]
pub fn f64_div_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f64>(|l, r| l / r)
}

#[allow(dead_code)]
pub fn f64_min_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f64>(|l, r| {
        if l.is_nan() {
            return l;
        }
        if r.is_nan() {
            return r;
        }
        if l == f64::NEG_INFINITY || r == f64::NEG_INFINITY {
            return f64::NEG_INFINITY;
        }
        if l == f64::INFINITY {
            return r;
        }
        if r == f64::INFINITY {
            return l;
        }
        if l == 0.0 && r == 0.0 {
            if l.is_sign_negative() {
                return l;
            }
            return r;
        }
        l.min(r)
    })
}

#[allow(dead_code)]
pub fn f64_max_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f64>(|l, r| {
        if l.is_nan() {
            return l;
        }
        if r.is_nan() {
            return r;
        }
        if l == f64::INFINITY || r == f64::INFINITY {
            return f64::INFINITY;
        }
        if l == f64::NEG_INFINITY {
            return r;
        }
        if r == f64::NEG_INFINITY {
            return l;
        }
        if l == 0.0 && r == 0.0 {
            if l.is_sign_positive() {
                return l;
            }
            return r;
        }
        l.max(r)
    })
}

#[allow(dead_code)]
pub fn f64_copysign_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.binop::<f64>(|l, r| l.copysign(r))
}

#[allow(dead_code)]
pub fn i32_wrap_i64_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<u64, u32>(|o| o as u32)
}

#[allow(dead_code)]
pub fn i32_trunc_f32_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop_trap::<f32, i32>(|o| {
        if o.is_nan() {
            return Err(TrapKind::InvalidConversionToInteger);
        }
        if o >= i32::MAX as f32 || o < i32::MIN as f32 {
            return Err(TrapKind::IntegerOverflow);
        }
        Ok(o as i32)
    })
}

#[allow(dead_code)]
pub fn i32_trunc_f32_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop_trap::<f32, u32>(|o| {
        if o.is_nan() {
            return Err(TrapKind::InvalidConversionToInteger);
        }
        if o >= u32::MAX as f32 || o <= -1f32 {
            return Err(TrapKind::IntegerOverflow);
        }
        Ok(o as u32)
    })
}

#[allow(dead_code)]
pub fn i32_trunc_f64_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop_trap::<f64, i32>(|o| {
        if o.is_nan() {
            return Err(TrapKind::InvalidConversionToInteger);
        }
        if o >= (i32::MAX as f64 + 1f64) || o <= (i32::MIN as f64 - 1f64) {
            return Err(TrapKind::IntegerOverflow);
        }
        Ok(o as i32)
    })
}

#[allow(dead_code)]
pub fn i32_trunc_f64_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop_trap::<f64, u32>(|o| {
        if o.is_nan() {
            return Err(TrapKind::InvalidConversionToInteger);
        }
        if o >= (u32::MAX as f64 + 1f64) || o <= -1f64 {
            return Err(TrapKind::IntegerOverflow);
        }
        Ok(o as u32)
    })
}

#[allow(dead_code)]
pub fn i64_extend_i32_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<i32, i64>(|o| o as i64)
}

#[allow(dead_code)]
pub fn i64_extend_i32_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<u32, u64>(|o| o as u64)
}

#[allow(dead_code)]
pub fn i64_trunc_f32_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop_trap::<f32, i64>(|o| {
        if o.is_nan() {
            return Err(TrapKind::InvalidConversionToInteger);
        }
        if o >= i64::MAX as f32 || o < i64::MIN as f32 {
            return Err(TrapKind::IntegerOverflow);
        }
        Ok(o as i64)
    })
}

#[allow(dead_code)]
pub fn i64_trunc_f32_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop_trap::<f32, u64>(|o| {
        if o.is_nan() {
            return Err(TrapKind::InvalidConversionToInteger);
        }
        if o >= u64::MAX as f32 || o <= -1f32 {
            return Err(TrapKind::IntegerOverflow);
        }
        Ok(o as u64)
    })
}

#[allow(dead_code)]
pub fn i64_trunc_f64_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop_trap::<f64, i64>(|o| {
        if o.is_nan() {
            return Err(TrapKind::InvalidConversionToInteger);
        }
        if o >= i64::MAX as f64 || o < i64::MIN as f64 {
            return Err(TrapKind::IntegerOverflow);
        }
        Ok(o as i64)
    })
}

#[allow(dead_code)]
pub fn i64_trunc_f64_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop_trap::<f64, u64>(|o| {
        if o.is_nan() {
            return Err(TrapKind::InvalidConversionToInteger);
        }
        if o >= u64::MAX as f64 || o <= -1f64 {
            return Err(TrapKind::IntegerOverflow);
        }
        Ok(o as u64)
    })
}

#[allow(dead_code)]
pub fn f32_convert_i32_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<i32, f32>(|o| o as f32)
}

#[allow(dead_code)]
pub fn f32_convert_i32_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<u32, f32>(|o| o as f32)
}

#[allow(dead_code)]
pub fn f32_convert_i64_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<i64, f32>(|o| o as f32)
}

#[allow(dead_code)]
pub fn f32_convert_i64_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<u64, f32>(|o| o as f32)
}

#[allow(dead_code)]
pub fn f32_demote_f64_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<f64, f32>(|o| o as f32)
}

#[allow(dead_code)]
pub fn f64_convert_i32_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<i32, f64>(|o| o as f64)
}

#[allow(dead_code)]
pub fn f64_convert_i32_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<u32, f64>(|o| o as f64)
}

#[allow(dead_code)]
pub fn f64_convert_i64_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<i64, f64>(|o| o as f64)
}

#[allow(dead_code)]
pub fn f64_convert_i64_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<u64, f64>(|o| o as f64)
}

#[allow(dead_code)]
pub fn f64_promote_f32_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<f32, f64>(|o| o as f64)
}

#[allow(dead_code)]
pub fn i32_reinterpret_f32_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<f32, u32>(|o| o.to_bits())
}

#[allow(dead_code)]
pub fn i64_reinterpret_f64_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<f64, u64>(|o| o.to_bits())
}

#[allow(dead_code)]
pub fn f32_reinterpret_i32_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<u32, f32>(f32::from_bits)
}

#[allow(dead_code)]
pub fn f64_reinterpret_i64_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<u64, f64>(f64::from_bits)
}

#[allow(dead_code)]
pub fn i32_extend8_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<u32>(|o| (o as u8 as i8 as u32))
}

#[allow(dead_code)]
pub fn i32_extend16_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<u32>(|o| (o as u16 as i16 as u32))
}

#[allow(dead_code)]
pub fn i64_extend8_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<u64>(|o| (o as u8 as i8 as u64))
}

#[allow(dead_code)]
pub fn i64_extend16_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<u64>(|o| (o as u16 as i16 as u64))
}

#[allow(dead_code)]
pub fn i64_extend32_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.unop::<u64>(|o| (o as u32 as i32 as u64))
}

#[allow(dead_code)]
pub fn ref_null_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let rt = _ec.op_reftype()?;
    _ec.push(Ref::Null(rt))
}

#[allow(dead_code)]
pub fn ref_is_null_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.testop::<Ref>(|i| i.is_null())
}

#[allow(dead_code)]
pub fn ref_func_exec(_ec: &mut ExecutionContext) -> Result<()> {
    let fi = _ec.op_u32()?;
    _ec.push_func_ref(fi)
}

#[allow(dead_code)]
pub fn i32_trunc_sat_f32_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<f32, i32>(|i| i as i32)
}

#[allow(dead_code)]
pub fn i32_trunc_sat_f32_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<f32, u32>(|i| i as u32)
}

#[allow(dead_code)]
pub fn i32_trunc_sat_f64_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<f64, i32>(|i| i as i32)
}

#[allow(dead_code)]
pub fn i32_trunc_sat_f64_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<f64, u32>(|i| i as u32)
}

#[allow(dead_code)]
pub fn i64_trunc_sat_f32_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<f32, i64>(|i| i as i64)
}

#[allow(dead_code)]
pub fn i64_trunc_sat_f32_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<f32, u64>(|i| i as u64)
}

#[allow(dead_code)]
pub fn i64_trunc_sat_f64_s_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<f64, i64>(|i| i as i64)
}

#[allow(dead_code)]
pub fn i64_trunc_sat_f64_u_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.convop::<f64, u64>(|i| i as u64)
}

#[allow(dead_code)]
pub fn memory_init_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.mem_init()
}

#[allow(dead_code)]
pub fn data_drop_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.data_drop()
}

#[allow(dead_code)]
pub fn memory_copy_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.mem_copy()
}

#[allow(dead_code)]
pub fn memory_fill_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.mem_fill()
}

#[allow(dead_code)]
pub fn table_init_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.table_init()
}

#[allow(dead_code)]
pub fn elem_drop_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.elem_drop()
}

#[allow(dead_code)]
pub fn table_copy_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.table_copy()
}

#[allow(dead_code)]
pub fn table_grow_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.table_grow()
}

#[allow(dead_code)]
pub fn table_size_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.table_size()
}

#[allow(dead_code)]
pub fn table_fill_exec(_ec: &mut ExecutionContext) -> Result<()> {
    _ec.table_fill()
}
