use {
    super::{
        error::{Result, RuntimeError, TrapKind},
        instance::{addr, addr::Address},
        values::{Ref, Value},
        Runtime,
    },
    crate::{
        impl_bug,
        instructions::{exec_method, op_consts},
        log_tag::Tag,
        runtime::{instance::MemInstance, stack::Label},
        syntax::{types::RefType, Opcode},
    },
    std::convert::{TryFrom, TryInto},
    wrausmt_common::{logger::Logger, true_or::TrueOr},
};

pub struct ExecutionContext<'l> {
    runtime: &'l mut Runtime,
    body:    &'l [u8],
    pc:      usize,
}

/// Passed to `push_label` to differentiate between blocks that return the param
/// types (loops) and all other normal blocks.
pub enum LabelType {
    Start,
    End,
}

pub type TrapResult<T> = std::result::Result<T, TrapKind>;
pub trait TryValue: TryFrom<Value, Error = RuntimeError> {}
pub trait TryIntoValue: TryValue + TryFrom<Value, Error = RuntimeError> + Into<Value> {}
macro_rules! value {
    ($($t:ty),+) => {
        $(
            impl TryValue for $t {}
            impl TryIntoValue for $t {}
        )+
    };
}
value!(i32, i64, u8, u32, u64, f32, f64, usize, Ref);

pub trait ExecutionContextActions {
    fn log(&self, tag: Tag, msg: impl Fn() -> String);
    fn skip(&mut self, bytes: usize);
    fn next_byte(&mut self) -> u8;
    fn op_u32(&mut self) -> Result<u32>;
    fn op_u64(&mut self) -> Result<u64>;
    fn op_reftype(&mut self) -> Result<RefType>;
    fn get_local(&mut self, locidx: u32) -> Result<Value>;
    fn set_local(&mut self, locidx: u32, val: Value) -> Result<()>;
    fn get_global(&mut self, gidx: u32) -> Result<Value>;
    fn set_global(&mut self, gidx: u32, val: Value) -> Result<()>;
    fn push_value(&mut self, val: Value) -> Result<()>;
    fn push_func_ref(&mut self, fidx: u32) -> Result<()>;
    fn push_label(&mut self, label_type: LabelType) -> Result<()>;
    fn push(&mut self, val: impl Into<Value>) -> Result<()>;
    fn pop_value(&mut self) -> Result<Value>;
    fn pop_label(&mut self) -> Result<Label>;
    fn pop<T: TryValue>(&mut self) -> Result<T>;
    fn call(&mut self, fidx: u32) -> Result<()>;
    fn call_addr(&mut self, addr: Address<addr::Function>, tyidx: u32) -> Result<()>;
    fn mem(&mut self, midx: u32) -> Result<&mut MemInstance>;
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
    fn get_func_table(&mut self, tidx: u32, eidx: u32) -> Result<u32>;
    fn get_table_elem(&mut self, tidx: u32, eidx: u32) -> Result<Ref>;
    fn set_table_elem(
        &mut self,
        tidx: u32,
        eidx: u32,
        val: impl TryInto<Ref, Error = RuntimeError>,
    ) -> Result<()>;
    fn elem_drop(&mut self) -> Result<()>;
    fn data_drop(&mut self) -> Result<()>;

    fn br(&mut self, labidx: u32) -> Result<()>;
    fn continuation(&mut self, cnt: u32) -> Result<()>;
    fn ret(&mut self) -> Result<()>;

    fn get_mem<const S: usize>(&mut self) -> Result<[u8; S]> {
        let _a = self.op_u32()?;
        let o = self.op_u32()?;
        let b = self.pop::<usize>()?;
        Ok(self
            .mem(0)?
            .read(o as usize, b, S)?
            .try_into()
            .map_err(|e| impl_bug!("conversion error {:?}", e))?)
    }

    fn put_mem<const S: usize>(&mut self, bytes: [u8; S]) -> Result<()> {
        let _a = self.op_u32()?;
        let o = self.op_u32()?;
        let b = self.pop::<usize>()?;
        self.mem(0)?.write(o as usize, b, &bytes)
    }

    fn binop<T: TryIntoValue>(&mut self, op: impl Fn(T, T) -> T) -> Result<()> {
        let r = self.pop::<T>()?;
        let l = self.pop::<T>()?;
        self.push(op(l, r))
    }

    fn binop_trap<T: TryIntoValue>(&mut self, op: impl Fn(T, T) -> TrapResult<T>) -> Result<()> {
        let r = self.pop::<T>()?;
        let l = self.pop::<T>()?;
        self.push(op(l, r)?)
    }

    fn convop<I: TryValue, O: Into<Value>>(&mut self, op: impl Fn(I) -> O) -> Result<()> {
        let i = self.pop::<I>()?;
        self.push(op(i))
    }

    fn convop_trap<I: TryValue, O: Into<Value>>(
        &mut self,
        op: impl Fn(I) -> TrapResult<O>,
    ) -> Result<()> {
        let i = self.pop::<I>()?;
        self.push(op(i)?)
    }

    fn unop<T: TryIntoValue>(&mut self, op: impl Fn(T) -> T) -> Result<()> {
        let o = self.pop::<T>()?;
        self.push(op(o))
    }

    fn relop<T: TryIntoValue>(&mut self, op: impl Fn(T, T) -> bool) -> Result<()> {
        let r = self.pop::<T>()?;
        let l = self.pop::<T>()?;
        self.push(if op(l, r) { 1 } else { 0 })
    }

    fn testop<T: TryIntoValue>(&mut self, op: impl Fn(T) -> bool) -> Result<()> {
        let i = self.pop::<T>()?;
        self.push(if op(i) { 1 } else { 0 })
    }
}

impl<'l> ExecutionContextActions for ExecutionContext<'l> {
    fn log(&self, tag: Tag, msg: impl Fn() -> String) {
        self.runtime.logger.log(tag, msg);
    }

    fn skip(&mut self, bytes: usize) {
        self.pc += bytes;
    }

    fn next_byte(&mut self) -> u8 {
        self.body[self.pc]
    }

    fn op_u32(&mut self) -> Result<u32> {
        let result = u32::from_le_bytes(
            self.body[self.pc..self.pc + 4]
                .try_into()
                .map_err(|e| impl_bug!("conversion error {e:?}"))?,
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
            _ => Err(impl_bug!("{byte} does not encode a ref type").into()),
        }
    }

    fn op_u64(&mut self) -> Result<u64> {
        let result = u64::from_le_bytes(
            self.body[self.pc..self.pc + 8]
                .try_into()
                .map_err(|e| impl_bug!("conversion error {e:?}"))?,
        );
        self.pc += 8;
        Ok(result)
    }

    fn get_local(&mut self, locidx: u32) -> Result<Value> {
        let val = self.runtime.stack.get_local(locidx);
        self.log(Tag::Local, || format!("GET {locidx} {val:?}"));
        val
    }

    fn set_local(&mut self, locidx: u32, val: Value) -> Result<()> {
        self.log(Tag::Local, || format!("SET {locidx} {val:?}"));
        self.runtime.stack.set_local(locidx, val)
    }

    fn get_table_elem(&mut self, tidx: u32, eidx: u32) -> Result<Ref> {
        let taddr = self.runtime.stack.active_module()?.table(tidx);
        let table = self.runtime.store.table(taddr)?;
        let elem =
            table
                .elem
                .get(eidx as usize)
                .copied()
                .ok_or(TrapKind::OutOfBoundsTableAccess(
                    eidx as usize,
                    table.elem.len(),
                ))?;
        Ok(elem)
    }

    fn set_table_elem(
        &mut self,
        tidx: u32,
        eidx: u32,
        val: impl TryInto<Ref, Error = RuntimeError>,
    ) -> Result<()> {
        let taddr = self.runtime.stack.active_module()?.table(tidx);
        let table = self.runtime.store.table_mut(taddr)?;
        match table.elem.get_mut(eidx as usize) {
            Some(e) => val.try_into().map(|v| *e = v),
            _ => Err(TrapKind::OutOfBoundsTableAccess(eidx as usize, table.elem.len()).into()),
        }
    }

    fn get_func_table(&mut self, tidx: u32, eidx: u32) -> Result<u32> {
        match self.get_table_elem(tidx, eidx)? {
            Ref::Func(a) => Ok(a.0),
            Ref::Null(RefType::Func) => Err(TrapKind::UninitializedElement.into()),
            e => Err(impl_bug!("not a func {e:?} FOR {tidx} {eidx}"))?,
        }
    }

    fn mem(&mut self, midx: u32) -> Result<&mut MemInstance> {
        let maddr = self.runtime.stack.active_module()?.mem(midx);
        self.log(Tag::Mem, || format!("USING MEM {maddr:?}"));
        self.runtime.store.mem_mut(maddr)
    }

    fn mem_init(&mut self) -> Result<()> {
        let didx = self.op_u32()?;
        let n = self.pop::<u32>()? as usize;
        let src = self.pop::<u32>()? as usize;
        let dst = self.pop::<u32>()? as usize;
        // TODO if s + n or d + n > sie of table 0, trap
        let maddr = self.runtime.stack.active_module()?.mem(0);
        let daddr = self.runtime.stack.active_module()?.data(didx);
        self.runtime
            .store
            .copy_data_to_mem(maddr, daddr, src, dst, n)
    }

    fn mem_size(&mut self) -> Result<()> {
        let maddr = self.runtime.stack.active_module()?.mem(0);
        let size = self.runtime.store.mem(maddr)?.size() as u32;
        self.runtime.stack.push_value(size.into());
        Ok(())
    }

    fn mem_grow(&mut self) -> Result<()> {
        let pgs = self.pop::<u32>()?;
        let maddr = self.runtime.stack.active_module()?.mem(0);
        let result = self.runtime.store.grow_mem(maddr, pgs)?;
        match result {
            None => self.push_value((-1i32).into()),
            Some(s) => self.push_value(s.into()),
        }
    }

    fn mem_fill(&mut self) -> Result<()> {
        let n = self.pop::<usize>()?;
        let val = self.pop::<u8>()?;
        let d = self.pop::<usize>()?;
        let maddr = self.runtime.stack.active_module()?.mem(0);
        // Note: the spec describes table fill as a recursive set of calls to table set
        // + table fill, we use a function here to emulate the same behavior with
        // less overhead.
        self.runtime.store.fill_mem(maddr, n, val, d)
    }

    fn mem_copy(&mut self) -> Result<()> {
        let n = self.pop::<usize>()?;
        let s = self.pop::<usize>()?;
        let d = self.pop::<usize>()?;
        let maddr = self.runtime.stack.active_module()?.mem(0);
        // Note: the spec describes table fill as a recursive set of calls to table set
        // + table fill, we use a function here to emulate the same behavior with
        // less overhead.
        self.runtime.store.copy_mem_to_mem(maddr, s, d, n)
    }

    fn br(&mut self, labidx: u32) -> Result<()> {
        let label = self.runtime.stack.break_to_label(labidx)?;
        self.pc = label.continuation as usize;
        Ok(())
    }

    fn ret(&mut self) -> Result<()> {
        self.pc = self.body.len() - 1;
        Ok(())
    }

    fn table_init(&mut self) -> Result<()> {
        let tidx = self.op_u32()?;
        let eidx = self.op_u32()?;
        let n = self.pop::<u32>()? as usize;
        let src = self.pop::<u32>()? as usize;
        let dst = self.pop::<u32>()? as usize;
        // TODO if s + n or d + n > sie of table 0, trap
        let taddr = self.runtime.stack.active_module()?.table(tidx);
        let eaddr = self.runtime.stack.active_module()?.elem(eidx);
        self.runtime
            .store
            .copy_elems_to_table(taddr, eaddr, src, dst, n)
    }

    fn table_copy(&mut self) -> Result<()> {
        let dstidx = self.op_u32()?;
        let srcidx = self.op_u32()?;
        let n = self.pop::<u32>()? as usize;
        let src = self.pop::<u32>()? as usize;
        let dst = self.pop::<u32>()? as usize;
        // TODO if s + n or d + n > sie of table 0, trap
        let dstaddr = self.runtime.stack.active_module()?.table(dstidx);
        let srcaddr = self.runtime.stack.active_module()?.table(srcidx);
        self.runtime
            .store
            .copy_table_to_table(dstaddr, srcaddr, dst, src, n)
    }

    fn table_size(&mut self) -> Result<()> {
        let tidx = self.op_u32()?;
        let taddr = self.runtime.stack.active_module()?.table(tidx);
        let size = self.runtime.store.table(taddr)?.elem.len() as u32;
        self.runtime.stack.push_value(size.into());
        Ok(())
    }

    fn table_grow(&mut self) -> Result<()> {
        let tidx = self.op_u32()?;
        let amt = self.pop::<u32>()?;
        let val = self.pop::<Ref>()?;
        let taddr = self.runtime.stack.active_module()?.table(tidx);
        let result = self.runtime.store.grow_table(taddr, amt, val)?;
        let result = match result {
            Some(result) => result as i32,
            None => -1,
        };
        self.runtime.stack.push_value(result.into());
        Ok(())
    }

    fn table_fill(&mut self) -> Result<()> {
        let tidx = self.op_u32()?;
        let n = self.pop::<usize>()?;
        let val = self.pop::<Ref>()?;
        let i = self.pop::<usize>()?;
        // TODO if i + n > sie of table 0, trap
        let taddr = self.runtime.stack.active_module()?.table(tidx);
        // Note: the spec describes table fill as a recursive set of calls to table set
        // + table fill, we use a function here to emulate the same behavior with
        // less overhead.
        self.runtime.store.fill_table(taddr, n, val, i)?;
        Ok(())
    }

    fn elem_drop(&mut self) -> Result<()> {
        let eidx = self.op_u32()?;
        let eaddr = self.runtime.stack.active_module()?.elem(eidx);
        self.runtime.store.elem_drop(eaddr)
    }

    fn data_drop(&mut self) -> Result<()> {
        let didx = self.op_u32()?;
        let daddr = self.runtime.stack.active_module()?.data(didx);
        self.runtime.store.data_drop(daddr)
    }

    fn continuation(&mut self, cnt: u32) -> Result<()> {
        self.pc = cnt as usize;
        self.log(Tag::Flow, || format!("CONTINUE AT {cnt:x}"));
        if self.pc >= self.body.len() {
            panic!(
                "invalid continuation {pc} for body size {size}",
                pc = self.pc,
                size = self.body.len()
            )
        }
        Ok(())
    }

    fn get_global(&mut self, gidx: u32) -> Result<Value> {
        self.runtime
            .store
            .global(self.runtime.stack.active_module()?.global(gidx))
    }

    fn set_global(&mut self, gidx: u32, val: Value) -> Result<()> {
        self.runtime
            .store
            .set_global(self.runtime.stack.active_module()?.global(gidx), val)
    }

    fn push_value(&mut self, val: Value) -> Result<()> {
        self.runtime.stack.push_value(val);
        Ok(())
    }

    fn push_func_ref(&mut self, fidx: u32) -> Result<()> {
        let faddr = self.runtime.stack.active_module()?.func(fidx);
        self.runtime.stack.push_value(Ref::Func(faddr).into());
        Ok(())
    }

    fn push_label(&mut self, label_type: LabelType) -> Result<()> {
        let param_arity = self.op_u32()?;
        let result_arity = self.op_u32()?;
        let result_arity = match label_type {
            LabelType::End => result_arity,
            LabelType::Start => param_arity,
        };
        let continuation = self.op_u32()?;
        self.runtime
            .stack
            .push_label(param_arity, result_arity, continuation)?;
        Ok(())
    }

    fn pop_label(&mut self) -> Result<Label> {
        self.runtime.stack.pop_label()
    }

    fn push(&mut self, val: impl Into<Value>) -> Result<()> {
        self.push_value(val.into())
    }

    fn pop_value(&mut self) -> Result<Value> {
        self.runtime.stack.pop_value()
    }

    fn pop<T: TryValue>(&mut self) -> Result<T> {
        let val = self.pop_value()?;
        val.try_into()
    }

    fn call(&mut self, fidx: u32) -> Result<()> {
        self.runtime
            .invoke_addr(self.runtime.stack.active_module()?.func(fidx))
    }

    fn call_addr(&mut self, addr: Address<addr::Function>, tyidx: u32) -> Result<()> {
        let funcinst = self.runtime.store.func(addr)?;
        let expected_type = self.runtime.stack.active_module()?.func_type(tyidx);
        (&funcinst.functype == expected_type)
            .true_or_else(|| TrapKind::CallIndirectTypeMismatch)?;
        self.runtime.invoke(funcinst)
    }
}

impl<'l> ExecutionContext<'l> {
    pub fn run(&mut self) -> Result<()> {
        while self.pc < self.body.len() {
            let op = self.body[self.pc];
            let opcode = match op {
                op_consts::EXTENDED_PREFIX => {
                    self.pc += 1;
                    Opcode::Extended(self.body[self.pc])
                }
                op_consts::SIMD_PREFIX => {
                    self.pc += 1;
                    Opcode::Simd(self.body[self.pc])
                }
                _ => Opcode::Normal(op),
            };
            self.log(Tag::Op, || format!("BEGIN 0x{opcode:x?}"));
            self.pc += 1;
            exec_method(opcode, self)?;
            self.log(Tag::Op, || format!("FINISHED 0x{opcode:x?}"));
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
    fn log(&self, tag: Tag, msg: impl Fn() -> String) {
        self.logger.log(tag, msg);
    }

    pub fn enter(&mut self, body: &[u8]) -> Result<()> {
        self.log(Tag::Enter, || {
            format!("ENTER EXPR {expr}", expr = Body(body))
        });
        let mut ic = ExecutionContext {
            runtime: self,
            body,
            pc: 0,
        };
        let result = ic.run();
        if let Err(ref e) = result {
            println!("UNWINDING FOR ERROR {e:?}");
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
            _ => Err(impl_bug!("non-ref result for expression"))?,
        }
    }
}
