use super::{
    values::{Ref, Value},
    Runtime,
};
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
    fn set_global(&mut self, idx: u32, val: Value) -> Result<()>;
    fn push_value(&mut self, val: Value) -> Result<()>;
    fn push_func_ref(&mut self, idx: u32) -> Result<()>;
    fn push_label_end(&mut self) -> Result<()>;
    fn push_label_start(&mut self) -> Result<()>;
    fn push<T: Into<Value>>(&mut self, val: T) -> Result<()>;
    fn pop_value(&mut self) -> Result<Value>;
    fn pop_label(&mut self) -> Result<Label>;
    fn pop<T: TryFrom<Value, Error = Error>>(&mut self) -> Result<T>;
    fn call(&mut self, idx: u32) -> Result<()>;
    fn mem(&mut self, idx: u32) -> Result<&mut MemInstance>;
    fn grow_mem(&mut self, pgs: u32) -> Result<Option<u32>>;
    fn table_init(&mut self) -> Result<()>;
    fn elem_drop(&mut self) -> Result<()>;
    fn mem_init(&mut self) -> Result<()>;
    fn data_drop(&mut self) -> Result<()>;

    fn br(&mut self, labelidx: u32) -> Result<()>;
    fn continuation(&mut self, cnt: u32) -> Result<()>;
    fn ret(&mut self) -> Result<()>;

    fn get_mem<const S: usize>(&mut self) -> Result<[u8; S]> {
        let _a = self.op_u32()?;
        let o = self.op_u32()?;
        let b = self.pop::<u32>()?;
        let i = (b + o) as usize;
        self.mem(0)?.data[i..i + S].try_into().wrap("into")
    }

    fn put_mem<const S: usize>(&mut self, bytes: [u8; S]) -> Result<()> {
        let _a = self.op_u32()?;
        let o = self.op_u32()?;
        let b = self.pop::<u32>()?;
        let m = self.mem(0)?;
        let i = (o + b) as usize;
        m.data[i..i + S].clone_from_slice(&bytes);
        Ok(())
    }

    fn binop<T, F>(&mut self, op: F) -> Result<()>
    where
        T: TryFrom<Value, Error = Error> + Into<Value>,
        F: Fn(T, T) -> T,
    {
        let r = self.pop::<T>()?;
        let l = self.pop::<T>()?;
        self.push(op(l, r))
    }

    fn convop<I, O, F>(&mut self, op: F) -> Result<()>
    where
        I: TryFrom<Value, Error = Error>,
        O: Into<Value>,
        F: Fn(I) -> O,
    {
        let i = self.pop::<I>()?;
        self.push(op(i))
    }

    fn unop<T, F>(&mut self, op: F) -> Result<()>
    where
        T: TryFrom<Value, Error = Error> + Into<Value>,
        F: Fn(T) -> T,
    {
        let o = self.pop::<T>()?;
        self.push(op(o))
    }

    fn testop<T, F>(&mut self, op: F) -> Result<()>
    where
        T: TryFrom<Value, Error = Error> + Into<Value>,
        F: Fn(T, T) -> bool,
    {
        let r = self.pop::<T>()?;
        let l = self.pop::<T>()?;
        self.push(if op(l, r) { 1 } else { 0 })
    }
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
        let val = self.runtime.stack.get_local(idx);
        self.log("LOCAL", || format!("GET {} {:?}", idx, val));
        val
    }

    fn set_local(&mut self, idx: u32, val: Value) -> Result<()> {
        self.log("LOCAL", || format!("SET {} {:?}", idx, val));
        self.runtime.stack.set_local(idx, val)
    }

    fn get_func_table(&mut self, tidx: u32, elemidx: u32) -> Result<u32> {
        let tableaddr = &self.runtime.stack.get_table_addr(tidx)?;
        let table = self.runtime.store.table(*tableaddr)?;
        match table.elem[elemidx as usize] {
            Ref::Func(a) => Ok(a as u32),
            _ => panic!("not a func"),
        }
    }

    fn mem(&mut self, idx: u32) -> Result<&mut MemInstance> {
        let memaddr = self.runtime.stack.get_mem_addr(idx)?;
        self.log("MEM", || format!("USING MEM {:?}", memaddr));
        self.runtime.store.mem(memaddr)
    }

    fn grow_mem(&mut self, pgs: u32) -> Result<Option<u32>> {
        let memaddr = self.runtime.stack.get_mem_addr(0)?;
        self.runtime.store.grow_mem(memaddr, pgs)
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
        let elemaddr = self.runtime.stack.get_elem_addr(elemidx)?;
        self.runtime
            .store
            .copy_elems_to_table(tableaddr, elemaddr, src, dst, n)
    }

    fn mem_init(&mut self) -> Result<()> {
        let dataidx = self.op_u32()?;
        let n = self.pop::<u32>()? as usize;
        let src = self.pop::<u32>()? as usize;
        let dst = self.pop::<u32>()? as usize;
        // TODO if s + n or d + n > sie of table 0, trap
        let memaddr = self.runtime.stack.get_mem_addr(0)?;
        let dataaddr = self.runtime.stack.get_data_addr(dataidx)?;
        self.runtime
            .store
            .copy_data_to_mem(memaddr, dataaddr, src, dst, n)
    }

    fn elem_drop(&mut self) -> Result<()> {
        let elemidx = self.op_u32()?;
        self.runtime.store.elem_drop(elemidx)
    }

    fn data_drop(&mut self) -> Result<()> {
        let dataidx = self.op_u32()?;
        self.runtime.store.data_drop(dataidx)
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

    fn set_global(&mut self, idx: u32, val: Value) -> Result<()> {
        self.runtime
            .store
            .set_global(self.runtime.stack.get_global_addr(idx)?, val)
    }

    fn push_value(&mut self, val: Value) -> Result<()> {
        self.runtime.stack.push_value(val);
        Ok(())
    }

    fn push_func_ref(&mut self, idx: u32) -> Result<()> {
        self.runtime.stack.push_value(Ref::Func(idx).into());
        Ok(())
    }

    fn push_label_start(&mut self) -> Result<()> {
        let param_arity = self.op_u32()?;
        let _result_arity = self.op_u32()?;
        let continuation = self.op_u32()?;
        self.runtime
            .stack
            // Subtle difference: when the continuation is the start of the block,
            // The "result" is considered to be the parameters.
            .push_label(param_arity, param_arity, continuation)?;
        Ok(())
    }

    fn push_label_end(&mut self) -> Result<()> {
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
        self.enter(body)
    }

    pub fn eval_expr(&mut self, body: &[u8]) -> Result<Value> {
        self.exec_expr(body)?;
        self.stack.pop_value()
    }

    pub fn eval_ref_expr(&mut self, body: &[u8]) -> Result<Ref> {
        match self.eval_expr(body)? {
            Value::Ref(r) => Ok(r),
            _ => err!("non-ref result for expression"),
        }
    }
}
