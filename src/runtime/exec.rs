use super::{values::Ref, values::Value, Runtime};
use crate::err;
use crate::instructions::exec_method;
use crate::logger::Logger;
use crate::runtime::instance::MemInstance;
use crate::{
    error::{Error, Result, ResultFrom},
    runtime::stack::Label,
};
use std::{borrow::Borrow, convert::TryFrom, convert::TryInto, fmt::Display, hash::Hash};

/// Emit an implementation for one of the memory load instructions.
/// $n - the name of the function to emit
/// $t - the return type of the functiona
/// $s - the type of the stored value
macro_rules! get_mem {
    ( $n:ident, $t:ty, $s:expr ) => {
        fn $n(&mut self) -> Result<$t> {
            let _a = self.op_u32()?;
            let o = self.op_u32()?;
            let b = self.pop::<u32>()?;
            let i = (b + o) as usize;
            let db: [u8; $s] = self.mem(0)?.data[i..i + $s].try_into().wrap("to array")?;
            self.log("MEM", || {
                format!("GET {} BYTES {:?} {}", $s, db, stringify!($t))
            });
            let val = <$t>::from_le_bytes(db);
            Ok(val)
        }
    };
}

macro_rules! set_mem {
    ( $n:ident, $t:ty, $st:ty, $s:expr ) => {
        fn $n(&mut self) -> Result<()> {
            let _a = self.op_u32()?;
            let o = self.op_u32()?;
            let val = self.pop::<$t>()? as $st;
            let b = self.pop::<u32>()?;
            let bs = val.to_le_bytes();
            self.log("MEM", || format!("SETTING {:?} AS {:?}", val, bs));
            let i = (o + b) as usize;
            let m = self.mem(0)?;
            if i + $s > m.data.len() {
                // TODO better error -> trap
                return err!("out of bounds {} {}", i, $s);
            }
            m.data[i..i + $s].clone_from_slice(&bs);
            Ok(())
        }
    };
}

pub trait ExecutionContextActions {
    fn log<S: Borrow<str> + Eq + Hash + Display, F>(&self, tag: S, msg: F)
    where
        F: Fn() -> String;
    fn op_u32(&mut self) -> Result<u32>;
    fn op_u64(&mut self) -> Result<u64>;
    fn get_local(&mut self, idx: u32) -> Result<Value>;
    fn set_local(&mut self, idx: u32, val: Value) -> Result<()>;

    fn get_func_table(&mut self, tidx: u32, elemidx: u32) -> Result<u32>;
    fn get_global(&mut self, idx: u32) -> Result<Value>;
    fn set_global(&mut self, idx: u32, val: Value) -> Result<()>;
    fn push_value(&mut self, val: Value) -> Result<()>;
    fn push_func_ref(&mut self, idx: u32) -> Result<()>;
    fn push_label(&mut self) -> Result<()>;
    fn push<T: Into<Value>>(&mut self, val: T) -> Result<()>;
    fn pop_value(&mut self) -> Result<Value>;
    fn pop_label(&mut self) -> Result<Label>;
    fn pop<T: TryFrom<Value, Error = Error>>(&mut self) -> Result<T>;
    fn docall(&mut self, idx: u32) -> Result<()>;
    fn mem(&mut self, idx: u32) -> Result<&mut MemInstance>;
    fn grow_mem(&mut self, pgs: u32) -> Result<Option<u32>>;
    fn table_init(&mut self) -> Result<()>;
    fn elem_drop(&mut self) -> Result<()>;
    fn mem_init(&mut self) -> Result<()>;
    fn data_drop(&mut self) -> Result<()>;

    fn br(&mut self, labelidx: u32) -> Result<()>;
    fn continuation(&mut self, cnt: u32) -> Result<()>;
    fn ret(&mut self) -> Result<()>;

    get_mem! { get_mem_i32, u32, 4 }
    get_mem! { get_mem_i32_8s, i8, 1 }
    get_mem! { get_mem_i32_8u, u8, 1 }
    get_mem! { get_mem_i32_16s, i16, 2 }
    get_mem! { get_mem_i32_16u, u16, 2 }
    get_mem! { get_mem_i64_8s, i8, 1 }
    get_mem! { get_mem_i64_8u, u8, 1 }
    get_mem! { get_mem_i64_16s, i16, 2 }
    get_mem! { get_mem_i64_16u, u16, 2 }
    get_mem! { get_mem_i64_32s, i32, 4 }
    get_mem! { get_mem_i64_32u, u32, 4 }
    get_mem! { get_mem_i64, u64, 8 }
    get_mem! { get_mem_f32, f32, 4 }
    get_mem! { get_mem_f64, f64, 8 }

    set_mem! { set_mem_i32, u32, u32, 4 }
    set_mem! { set_mem_i32_8, u32, u8, 1 }
    set_mem! { set_mem_i32_16, u32, u16, 2 }
    set_mem! { set_mem_i64, u64, u64, 8 }
    set_mem! { set_mem_i64_8, u64, u8, 1 }
    set_mem! { set_mem_i64_16, u64, u16, 2 }
    set_mem! { set_mem_i64_32, u64, u32, 4 }
    set_mem! { set_mem_f32, f32, f32, 4 }
    set_mem! { set_mem_f64, f64, f64, 8 }
}

impl ExecutionContextActions for Runtime {
    fn log<S: Borrow<str> + Eq + Hash + Display, F>(&self, tag: S, msg: F)
    where
        F: Fn() -> String,
    {
        self.logger.log(tag, msg);
    }

    fn op_u32(&mut self) -> Result<u32> {
        self.callstack.next_u32()
    }

    fn op_u64(&mut self) -> Result<u64> {
        self.callstack.next_u64()
    }

    fn get_local(&mut self, idx: u32) -> Result<Value> {
        self.log("LOCAL", || format!("GET {}", idx));
        self.stack.get_local(idx)
    }

    fn set_local(&mut self, idx: u32, val: Value) -> Result<()> {
        self.stack.set_local(idx, val)
    }

    fn get_func_table(&mut self, tidx: u32, elemidx: u32) -> Result<u32> {
        let tableaddr = &self.stack.get_table_addr(tidx)?;
        let table = self.store.table(*tableaddr)?;
        match table.elem[elemidx as usize] {
            Ref::Func(a) => Ok(a as u32),
            _ => panic!("not a func"),
        }
    }

    fn mem(&mut self, idx: u32) -> Result<&mut MemInstance> {
        let memaddr = self.stack.get_mem_addr(idx)?;
        self.log("MEM", || format!("USING MEM {:?}", memaddr));
        self.store.mem(memaddr)
    }

    fn grow_mem(&mut self, pgs: u32) -> Result<Option<u32>> {
        let memaddr = self.stack.get_mem_addr(0)?;
        self.store.grow_mem(memaddr, pgs)
    }

    fn br(&mut self, labelidx: u32) -> Result<()> {
        let label = self.stack.break_to_label(labelidx)?;
        self.callstack.br(label.continuation as usize)?;
        Ok(())
    }

    fn table_init(&mut self) -> Result<()> {
        let tableidx = self.op_u32()?;
        let elemidx = self.op_u32()?;
        let n = self.pop::<u32>()? as usize;
        let src = self.pop::<u32>()? as usize;
        let dst = self.pop::<u32>()? as usize;
        // TODO if s + n or d + n > sie of table 0, trap
        let tableaddr = self.stack.get_table_addr(tableidx)?;
        let elemaddr = self.stack.get_elem_addr(elemidx)?;
        self.store
            .copy_elems_to_table(tableaddr, elemaddr, src, dst, n)
    }

    fn mem_init(&mut self) -> Result<()> {
        let dataidx = self.op_u32()?;
        let n = self.pop::<u32>()? as usize;
        let src = self.pop::<u32>()? as usize;
        let dst = self.pop::<u32>()? as usize;
        // TODO if s + n or d + n > sie of table 0, trap
        let memaddr = self.stack.get_mem_addr(0)?;
        let dataaddr = self.stack.get_data_addr(dataidx)?;
        self.store.copy_data_to_mem(memaddr, dataaddr, src, dst, n)
    }

    fn elem_drop(&mut self) -> Result<()> {
        let elemidx = self.op_u32()?;
        self.store.elem_drop(elemidx)
    }

    fn data_drop(&mut self) -> Result<()> {
        let dataidx = self.op_u32()?;
        self.store.data_drop(dataidx)
    }

    fn continuation(&mut self, cnt: u32) -> Result<()> {
        self.callstack.br(cnt as usize)?;
        Ok(())
    }

    fn get_global(&mut self, idx: u32) -> Result<Value> {
        self.store.global(self.stack.get_global_addr(idx)?)
    }

    fn set_global(&mut self, idx: u32, val: Value) -> Result<()> {
        self.store.set_global(self.stack.get_global_addr(idx)?, val)
    }

    fn push_value(&mut self, val: Value) -> Result<()> {
        self.stack.push_value(val);
        Ok(())
    }

    fn push_func_ref(&mut self, idx: u32) -> Result<()> {
        self.stack.push_value(Ref::Func(idx).into());
        Ok(())
    }

    fn push_label(&mut self) -> Result<()> {
        let param_arity = self.op_u32()?;
        let result_arity = self.op_u32()?;
        let continuation = self.op_u32()?;
        self.stack
            .push_label(param_arity, result_arity, continuation)?;
        Ok(())
    }

    fn pop_label(&mut self) -> Result<Label> {
        self.stack.pop_label()
    }

    fn push<T: Into<Value>>(&mut self, val: T) -> Result<()> {
        self.push_value(val.into())
    }

    fn pop_value(&mut self) -> Result<Value> {
        self.stack.pop_value()
    }

    fn pop<T: TryFrom<Value, Error = Error>>(&mut self) -> Result<T> {
        let val = self.pop_value()?;
        val.try_into().wrap("pop convert")
    }

    fn docall(&mut self, idx: u32) -> Result<()> {
        self.invoke(self.stack.get_function_addr(idx)?)
    }

    fn ret(&mut self) -> Result<()> {
        self.callstack.ret()?;
        self.stack.pop_activation()
    }
}

/// Implementation of instruction implementation for this runtime.
impl Runtime {
    pub fn run(&mut self) -> Result<()> {
        while let Ok(Some(op)) = self.callstack.next_op() {
            self.log("OP", || format!("BEGIN 0x{:x}", op));
            exec_method(op, self)?;
            self.log("OP", || format!("FINISHED 0x{:x}", op));
        }
        Ok(())
    }

    pub fn exec_expr(&mut self, body: Box<[u8]>) -> Result<()> {
        // Push a label so the end has something to pop
        self.stack.push_label(0, 1, body.len() as u32 - 1)?;
        self.callstack.eval(body);
        self.run()
    }

    pub fn eval_expr(&mut self, body: Box<[u8]>) -> Result<Value> {
        self.exec_expr(body)?;
        self.stack.pop_value()
    }

    pub fn eval_ref_expr(&mut self, body: Box<[u8]>) -> Result<Ref> {
        self.exec_expr(body)?;
        // Push a label so the end has something to pop
        let result = match self.stack.pop_value()? {
            Value::Ref(r) => Ok(r),
            _ => return err!("non-ref result for expression"),
        };
        result
    }
}
