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

pub struct ExecutionContext<'l> {
    runtime: &'l mut Runtime,
    body: &'l [u8],
    pc: usize,
}

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
    fn next_byte(&mut self) -> u8;
    fn op_u32(&mut self) -> Result<u32>;
    fn op_u64(&mut self) -> Result<u64>;
    fn get_local(&mut self, idx: u32) -> Result<Value>;
    fn set_local(&mut self, idx: u32, val: Value) -> Result<()>;

    fn get_func_table(&mut self, tidx: u32, elemidx: u32) -> Result<u32>;
    fn get_global(&mut self, idx: u32) -> Result<Value>;
    fn push_value(&mut self, val: Value) -> Result<()>;
    fn push_func_ref(&mut self, idx: u32) -> Result<()>;
    fn push_label(&mut self) -> Result<()>;
    fn push<T: Into<Value>>(&mut self, val: T) -> Result<()>;
    fn pop_value(&mut self) -> Result<Value>;
    fn pop_label(&mut self) -> Result<Label>;
    fn pop<T: TryFrom<Value, Error = Error>>(&mut self) -> Result<T>;
    fn call(&mut self, idx: u32) -> Result<()>;
    fn mem(&mut self, idx: u32) -> Result<&mut MemInstance>;
    fn table_init(&mut self) -> Result<()>;
    fn elem_drop(&mut self) -> Result<()>;

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

impl<'l> ExecutionContextActions for ExecutionContext<'l> {
    fn log<S: Borrow<str> + Eq + Hash + Display, F>(&self, tag: S, msg: F)
    where
        F: Fn() -> String,
    {
        self.runtime.logger.log(tag, msg);
    }

    fn next_byte(&mut self) -> u8 {
        self.body[self.pc]
    }

    fn op_u32(&mut self) -> Result<u32> {
        let result = u32::from_le_bytes(self.body[self.pc..self.pc + 4].try_into().wrap("idx")?);
        self.pc += 4;
        Ok(result)
    }

    fn op_u64(&mut self) -> Result<u64> {
        let result = u64::from_le_bytes(self.body[self.pc..self.pc + 8].try_into().wrap("idx")?);
        self.pc += 8;
        Ok(result)
    }

    fn get_local(&mut self, idx: u32) -> Result<Value> {
        self.log("LOCAL", || format!("GET {}", idx));
        self.runtime.stack.get_local(idx)
    }

    fn set_local(&mut self, idx: u32, val: Value) -> Result<()> {
        self.runtime.stack.set_local(idx, val)
    }

    fn get_func_table(&mut self, tidx: u32, elemidx: u32) -> Result<u32> {
        let table = &self.runtime.store.tables[tidx as usize];
        match table.elem[elemidx as usize] {
            Ref::Func(a) => Ok(a as u32),
            _ => panic!("not a func"),
        }
    }

    fn mem(&mut self, idx: u32) -> Result<&mut MemInstance> {
        self.runtime.store.mem(idx)
    }

    fn br(&mut self, labelidx: u32) -> Result<()> {
        let label = self.runtime.stack.break_to_label(labelidx)?;
        self.pc = label.continuation as usize;
        Ok(())
    }

    fn ret(&mut self) -> Result<()> {
        self.pc = self.body.len() - 1;
        Ok(())
    }

    fn table_init(&mut self) -> Result<()> {
        let tableidx = self.op_u32()?;
        let elemidx = self.op_u32()?;
        let n = self.pop::<u32>()? as usize;
        let src = self.pop::<u32>()? as usize;
        let dst = self.pop::<u32>()? as usize;
        // TODO if s + n or d + n > sie of table 0, trap
        let tableaddr = self.runtime.stack.get_table_addr(tableidx)?;
        let elemaddr = self.runtime.stack.get_table_addr(elemidx)?;
        self.runtime
            .store
            .copy_elems_to_table(tableaddr, elemaddr, src, dst, n)
    }

    fn elem_drop(&mut self) -> Result<()> {
        let elemidx = self.op_u32()?;
        self.runtime.store.elem_drop(elemidx)
    }

    fn continuation(&mut self, cnt: u32) -> Result<()> {
        self.pc = cnt as usize;
        self.log("FLOW", || format!("CONTINUE AT {:x}", cnt));
        if self.pc >= self.body.len() {
            panic!(
                "invalid continuation {} for body size {}",
                self.pc,
                self.body.len()
            )
        }
        Ok(())
    }

    fn get_global(&mut self, idx: u32) -> Result<Value> {
        self.runtime
            .store
            .global(self.runtime.stack.get_global_addr(idx)?)
    }

    fn push_value(&mut self, val: Value) -> Result<()> {
        self.runtime.stack.push_value(val);
        Ok(())
    }

    fn push_func_ref(&mut self, idx: u32) -> Result<()> {
        self.runtime.stack.push_value(Ref::Func(idx as u64).into());
        Ok(())
    }

    fn push_label(&mut self) -> Result<()> {
        let param_arity = self.op_u32()?;
        let result_arity = self.op_u32()?;
        let continuation = self.op_u32()?;
        self.runtime
            .stack
            .push_label(param_arity, result_arity, continuation)?;
        Ok(())
    }

    fn pop_label(&mut self) -> Result<Label> {
        self.runtime.stack.pop_label()
    }

    fn push<T: Into<Value>>(&mut self, val: T) -> Result<()> {
        self.push_value(val.into())
    }

    fn pop_value(&mut self) -> Result<Value> {
        self.runtime.stack.pop_value()
    }

    fn pop<T: TryFrom<Value, Error = Error>>(&mut self) -> Result<T> {
        let val = self.pop_value()?;
        val.try_into().wrap("pop convert")
    }

    fn call(&mut self, idx: u32) -> Result<()> {
        self.runtime
            .invoke(self.runtime.stack.get_function_addr(idx)?)
    }
}

impl<'l> ExecutionContext<'l> {
    pub fn run(&mut self) -> Result<()> {
        while self.pc < self.body.len() {
            let op = self.body[self.pc];
            self.log("OP", || format!("BEGIN 0x{:x}", op));
            self.pc += 1;
            exec_method(op, self)?;
            self.log("OP", || format!("FINISHED 0x{:x}", op));
        }
        Ok(())
    }
}

/// Implementation of instruction implementation for this runtime.
impl Runtime {
    fn log<S: Borrow<str> + Eq + Hash + Display, F: Fn() -> String>(&self, tag: S, msg: F) {
        self.logger.log(tag, msg);
    }

    pub fn enter(&mut self, body: &[u8]) -> Result<()> {
        self.log("ENTER", || format!("ENTER EXPR {:x?}", body));
        let mut ic = ExecutionContext {
            runtime: self,
            body,
            pc: 0,
        };
        ic.run()
    }

    pub fn exec_expr(&mut self, body: &[u8]) -> Result<()> {
        self.stack.push_label(0, 1, body.len() as u32 - 1)?;
        self.enter(body)
    }

    pub fn eval_expr(&mut self, body: &[u8]) -> Result<Value> {
        self.exec_expr(body)?;
        self.stack.pop_value()
    }

    pub fn eval_ref_expr(&mut self, body: &[u8]) -> Result<Ref> {
        self.exec_expr(body)?;
        match self.stack.pop_value()? {
            Value::Ref(r) => Ok(r),
            _ => err!("non-ref result for expression"),
        }
    }
}
