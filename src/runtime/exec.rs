use super::{
    error::RuntimeError,
    values::{Ref, Value},
    Runtime,
};
use super::{
    error::{Result, TrapKind},
    store::addr,
};
use crate::runtime::instance::MemInstance;
use crate::runtime::stack::Label;
use crate::{impl_bug, instructions::exec_method};
use crate::{logger::Logger, logger::Tag, types::RefType};
use std::{convert::TryFrom, convert::TryInto};

pub struct ExecutionContext<'l> {
    runtime: &'l mut Runtime,
    body: &'l [u8],
    pc: usize,
}

pub trait ExecutionContextActions {
    fn log<F>(&self, tag: Tag, msg: F)
    where
        F: Fn() -> String;
    fn next_byte(&mut self) -> u8;
    fn op_u32(&mut self) -> Result<u32>;
    fn op_u64(&mut self) -> Result<u64>;
    fn op_reftype(&mut self) -> Result<RefType>;
    fn get_local(&mut self, idx: u32) -> Result<Value>;
    fn set_local(&mut self, idx: u32, val: Value) -> Result<()>;

    fn get_global(&mut self, idx: u32) -> Result<Value>;
    fn set_global(&mut self, idx: u32, val: Value) -> Result<()>;
    fn push_value(&mut self, val: Value) -> Result<()>;
    fn push_func_ref(&mut self, idx: u32) -> Result<()>;
    fn push_label_end(&mut self) -> Result<()>;
    fn push_label_start(&mut self) -> Result<()>;
    fn push<T: Into<Value>>(&mut self, val: T) -> Result<()>;
    fn pop_value(&mut self) -> Result<Value>;
    fn pop_label(&mut self) -> Result<Label>;
    fn pop<T: TryFrom<Value, Error = RuntimeError>>(&mut self) -> Result<T>;
    fn call(&mut self, idx: u32) -> Result<()>;
    fn call_addr(&mut self, addr: addr::FuncAddr, typeidx: u32) -> Result<()>;
    fn mem(&mut self, idx: u32) -> Result<&mut MemInstance>;
    fn mem_init(&mut self) -> Result<()>;
    fn mem_size(&mut self) -> Result<()>;
    fn mem_grow(&mut self) -> Result<()>;
    fn mem_fill(&mut self) -> Result<()>;
    fn mem_copy(&mut self) -> Result<()>;
    fn table_init(&mut self) -> Result<()>;
    fn table_size(&mut self) -> Result<()>;
    fn table_grow(&mut self) -> Result<()>;
    fn table_fill(&mut self) -> Result<()>;
    fn table_copy(&mut self) -> Result<()>;
    fn get_func_table(&mut self, tidx: u32, elemidx: u32) -> Result<u32>;
    fn get_table_elem(&mut self, tidx: u32, eidx: u32) -> Result<Ref>;
    fn set_table_elem<V: TryInto<Ref, Error = RuntimeError>>(
        &mut self,
        tidx: u32,
        eidx: u32,
        val: V,
    ) -> Result<()>;
    fn elem_drop(&mut self) -> Result<()>;
    fn data_drop(&mut self) -> Result<()>;

    fn br(&mut self, labelidx: u32) -> Result<()>;
    fn continuation(&mut self, cnt: u32) -> Result<()>;
    fn ret(&mut self) -> Result<()>;

    fn get_mem<const S: usize>(&mut self) -> Result<[u8; S]> {
        let _a = self.op_u32()?;
        let o = self.op_u32()?;
        let b = self.pop::<usize>()?;
        self.mem(0)?
            .read(o as usize, b, S)?
            .try_into()
            .map_err(|e| impl_bug!("conversion error {:?}", e))
    }

    fn put_mem<const S: usize>(&mut self, bytes: [u8; S]) -> Result<()> {
        let _a = self.op_u32()?;
        let o = self.op_u32()?;
        let b = self.pop::<usize>()?;
        self.mem(0)?.write(o as usize, b, &bytes)
    }

    fn binop<T, F>(&mut self, op: F) -> Result<()>
    where
        T: TryFrom<Value, Error = RuntimeError> + Into<Value>,
        F: Fn(T, T) -> T,
    {
        let r = self.pop::<T>()?;
        let l = self.pop::<T>()?;
        self.push(op(l, r))
    }

    fn binop_trap<T, F>(&mut self, op: F) -> Result<()>
    where
        T: TryFrom<Value, Error = RuntimeError> + Into<Value> + Default + PartialEq,
        F: Fn(T, T) -> std::result::Result<T, TrapKind>,
    {
        let r = self.pop::<T>()?;
        let l = self.pop::<T>()?;
        self.push(op(l, r)?)
    }

    fn convop<I, O, F>(&mut self, op: F) -> Result<()>
    where
        I: TryFrom<Value, Error = RuntimeError>,
        O: Into<Value>,
        F: Fn(I) -> O,
    {
        let i = self.pop::<I>()?;
        self.push(op(i))
    }

    fn convop_trap<I, O, F>(&mut self, op: F) -> Result<()>
    where
        I: TryFrom<Value, Error = RuntimeError>,
        O: Into<Value>,
        F: Fn(I) -> std::result::Result<O, TrapKind>,
    {
        let i = self.pop::<I>()?;
        self.push(op(i)?)
    }

    fn unop<T, F>(&mut self, op: F) -> Result<()>
    where
        T: TryFrom<Value, Error = RuntimeError> + Into<Value>,
        F: Fn(T) -> T,
    {
        let o = self.pop::<T>()?;
        self.push(op(o))
    }

    fn relop<T, F>(&mut self, op: F) -> Result<()>
    where
        T: TryFrom<Value, Error = RuntimeError> + Into<Value>,
        F: Fn(T, T) -> bool,
    {
        let r = self.pop::<T>()?;
        let l = self.pop::<T>()?;
        self.push(if op(l, r) { 1 } else { 0 })
    }

    fn testop<T, F>(&mut self, op: F) -> Result<()>
    where
        T: TryFrom<Value, Error = RuntimeError> + Into<Value>,
        F: Fn(T) -> bool,
    {
        let i = self.pop::<T>()?;
        self.push(if op(i) { 1 } else { 0 })
    }
}

impl<'l> ExecutionContextActions for ExecutionContext<'l> {
    fn log<F: Fn() -> String>(&self, tag: Tag, msg: F) {
        self.runtime.logger.log(tag, msg);
    }

    fn next_byte(&mut self) -> u8 {
        self.body[self.pc]
    }

    fn op_u32(&mut self) -> Result<u32> {
        let result = u32::from_le_bytes(
            self.body[self.pc..self.pc + 4]
                .try_into()
                .map_err(|e| impl_bug!("conversion error {:?}", e))?,
        );
        self.pc += 4;
        Ok(result)
    }

    fn op_reftype(&mut self) -> Result<RefType> {
        let byte = self.body[self.pc];
        self.pc += 1;
        // Use the binary format encoding of ref type.
        match byte {
            0x70 => Ok(RefType::Func),
            0x6F => Ok(RefType::Extern),
            _ => Err(impl_bug!("{} does not encode a ref type", byte)),
        }
    }

    fn op_u64(&mut self) -> Result<u64> {
        let result = u64::from_le_bytes(
            self.body[self.pc..self.pc + 8]
                .try_into()
                .map_err(|e| impl_bug!("conversion error {:?}", e))?,
        );
        self.pc += 8;
        Ok(result)
    }

    fn get_local(&mut self, idx: u32) -> Result<Value> {
        let val = self.runtime.stack.get_local(idx);
        self.log(Tag::Local, || format!("GET {} {:?}", idx, val));
        val
    }

    fn set_local(&mut self, idx: u32, val: Value) -> Result<()> {
        self.log(Tag::Local, || format!("SET {} {:?}", idx, val));
        self.runtime.stack.set_local(idx, val)
    }

    fn get_table_elem(&mut self, tidx: u32, elemidx: u32) -> Result<Ref> {
        let tableaddr = self.runtime.stack.get_table_addr(tidx)?;
        let table = self.runtime.store.table(tableaddr)?;
        let elem = table
            .elem
            .get(elemidx as usize)
            .copied()
            .ok_or(TrapKind::OutOfBoundsTableAccess)?;
        Ok(elem)
    }

    fn set_table_elem<V: TryInto<Ref, Error = RuntimeError>>(
        &mut self,
        tidx: u32,
        elemidx: u32,
        val: V,
    ) -> Result<()> {
        let tableaddr = &self.runtime.stack.get_table_addr(tidx)?;
        let table = self.runtime.store.table_mut(*tableaddr)?;
        let elem = table
            .elem
            .get_mut(elemidx as usize)
            .ok_or(TrapKind::OutOfBoundsTableAccess)?;
        *elem = val.try_into()?;
        Ok(())
    }

    fn get_func_table(&mut self, tidx: u32, elemidx: u32) -> Result<u32> {
        match self.get_table_elem(tidx, elemidx)? {
            Ref::Func(a) => Ok(a),
            Ref::Null(RefType::Func) => Err(TrapKind::UninitializedElement.into()),
            e => Err(impl_bug!("not a func {:?} FOR {} {}", e, tidx, elemidx)),
        }
    }

    fn mem(&mut self, idx: u32) -> Result<&mut MemInstance> {
        let memaddr = self.runtime.stack.get_mem_addr(idx)?;
        self.log(Tag::Mem, || format!("USING MEM {:?}", memaddr));
        self.runtime.store.mem(memaddr)
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

    fn mem_size(&mut self) -> Result<()> {
        let memaddr = self.runtime.stack.get_mem_addr(0)?;
        let size = self.runtime.store.mem(memaddr)?.size() as u32;
        self.runtime.stack.push_value(size.into());
        Ok(())
    }

    fn mem_grow(&mut self) -> Result<()> {
        let pgs = self.pop::<u32>()?;
        let memaddr = self.runtime.stack.get_mem_addr(0)?;
        let result = self.runtime.store.grow_mem(memaddr, pgs)?;
        match result {
            None => self.push_value((-1i32).into()),
            Some(s) => self.push_value(s.into()),
        }
    }

    fn mem_fill(&mut self) -> Result<()> {
        let n = self.pop::<usize>()?;
        let val = self.pop::<u8>()?;
        let d = self.pop::<usize>()?;
        let memaddr = self.runtime.stack.get_mem_addr(0)?;
        // Note: the spec describes table fill as a recursive set of calls to table set + table
        // fill, we use a function here to emulate the same behavior with less overhead.
        self.runtime.store.fill_mem(memaddr, n, val, d)
    }

    fn mem_copy(&mut self) -> Result<()> {
        let n = self.pop::<usize>()?;
        let s = self.pop::<usize>()?;
        let d = self.pop::<usize>()?;
        let memaddr = self.runtime.stack.get_mem_addr(0)?;
        // Note: the spec describes table fill as a recursive set of calls to table set + table
        // fill, we use a function here to emulate the same behavior with less overhead.
        self.runtime.store.copy_mem_to_mem(memaddr, s, d, n)
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

    fn table_copy(&mut self) -> Result<()> {
        let dstidx = self.op_u32()?;
        let srcidx = self.op_u32()?;
        let n = self.pop::<u32>()? as usize;
        let src = self.pop::<u32>()? as usize;
        let dst = self.pop::<u32>()? as usize;
        // TODO if s + n or d + n > sie of table 0, trap
        let dstaddr = self.runtime.stack.get_table_addr(dstidx)?;
        let srcaddr = self.runtime.stack.get_table_addr(srcidx)?;
        self.runtime
            .store
            .copy_table_to_table(dstaddr, srcaddr, dst, src, n)
    }

    fn table_size(&mut self) -> Result<()> {
        let tabidx = self.op_u32()?;
        let tabaddr = self.runtime.stack.get_table_addr(tabidx)?;
        let size = self.runtime.store.table(tabaddr)?.elem.len() as u32;
        self.runtime.stack.push_value(size.into());
        Ok(())
    }

    fn table_grow(&mut self) -> Result<()> {
        let tabidx = self.op_u32()?;
        let amt = self.pop::<u32>()?;
        let val = self.pop::<Ref>()?;
        let tabaddr = self.runtime.stack.get_table_addr(tabidx)?;
        let result = self.runtime.store.grow_table(tabaddr, amt, val)?;
        let result = match result {
            Some(result) => result as i32,
            None => -1,
        };
        self.runtime.stack.push_value(result.into());
        Ok(())
    }

    fn table_fill(&mut self) -> Result<()> {
        let tabidx = self.op_u32()?;
        let n = self.pop::<usize>()?;
        let val = self.pop::<Ref>()?;
        let i = self.pop::<usize>()?;
        // TODO if i + n > sie of table 0, trap
        let tabaddr = self.runtime.stack.get_table_addr(tabidx)?;
        // Note: the spec describes table fill as a recursive set of calls to table set + table
        // fill, we use a function here to emulate the same behavior with less overhead.
        self.runtime.store.fill_table(tabaddr, n, val, i)?;
        Ok(())
    }

    fn elem_drop(&mut self) -> Result<()> {
        let elemidx = self.op_u32()?;
        let elemaddr = self.runtime.stack.get_elem_addr(elemidx)?;
        self.runtime.store.elem_drop(elemaddr)
    }

    fn data_drop(&mut self) -> Result<()> {
        let dataidx = self.op_u32()?;
        let dataaddr = self.runtime.stack.get_data_addr(dataidx)?;
        self.runtime.store.data_drop(dataaddr)
    }

    fn continuation(&mut self, cnt: u32) -> Result<()> {
        self.pc = cnt as usize;
        self.log(Tag::Flow, || format!("CONTINUE AT {:x}", cnt));
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
        let fa = self.runtime.stack.get_function_addr(idx)?;
        self.runtime.stack.push_value(Ref::Func(fa).into());
        Ok(())
    }

    fn push_label_start(&mut self) -> Result<()> {
        let typeidx = self.op_u32()?;
        let continuation = self.op_u32()?;
        let (param_arity, result_arity) = {
            let functype = self.runtime.stack.get_func_type(typeidx)?;
            (functype.params.len() as u32, functype.params.len() as u32)
        };
        self.runtime
            .stack
            .push_label(param_arity, result_arity, continuation)?;
        Ok(())
    }

    fn push_label_end(&mut self) -> Result<()> {
        let typeidx = self.op_u32()?;
        let continuation = self.op_u32()?;
        let (param_arity, result_arity) = {
            let functype = self.runtime.stack.get_func_type(typeidx)?;
            (functype.params.len() as u32, functype.result.len() as u32)
        };
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

    fn pop<T: TryFrom<Value, Error = RuntimeError>>(&mut self) -> Result<T> {
        let val = self.pop_value()?;
        val.try_into()
    }

    fn call(&mut self, idx: u32) -> Result<()> {
        self.runtime
            .invoke_addr(self.runtime.stack.get_function_addr(idx)?)
    }

    fn call_addr(&mut self, addr: addr::FuncAddr, typeidx: u32) -> Result<()> {
        let funcinst = self.runtime.store.func(addr)?;
        let expected_type = self.runtime.stack.get_func_type(typeidx)?;
        if &funcinst.functype != expected_type {
            return Err(TrapKind::CallIndirectTypeMismatch.into());
        }
        self.runtime.invoke(funcinst)
    }
}

impl<'l> ExecutionContext<'l> {
    pub fn run(&mut self) -> Result<()> {
        while self.pc < self.body.len() {
            let op = self.body[self.pc];
            self.log(Tag::Op, || format!("BEGIN 0x{:x}", op));
            self.pc += 1;
            exec_method(op, self)?;
            self.log(Tag::Op, || format!("FINISHED 0x{:x}", op));
        }
        Ok(())
    }
}

struct Body<'a>(&'a [u8]);

impl<'a> std::fmt::Display for Body<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut idx = 0usize;
        while idx < self.0.len() {
            let end = std::cmp::min(self.0.len(), idx + 16);
            writeln!(f, "{:02x?}", &self.0[idx..end])?;
            idx = end;
        }
        Ok(())
    }
}

/// Implementation of instruction implementation for this runtime.
impl Runtime {
    fn log<F: Fn() -> String>(&self, tag: Tag, msg: F) {
        self.logger.log(tag, msg);
    }

    pub fn enter(&mut self, body: &[u8]) -> Result<()> {
        self.log(Tag::Enter, || format!("ENTER EXPR {}", Body(body)));
        let mut ic = ExecutionContext {
            runtime: self,
            body,
            pc: 0,
        };
        let result = ic.run();
        if let Err(ref e) = result {
            println!("UNWINDING FOR ERROR {:?}", e);
            self.stack.unwind();
        }
        result
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
            _ => Err(impl_bug!("non-ref result for expression")),
        }
    }
}
