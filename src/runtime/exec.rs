use super::{ActivationFrame, values::Value, values::Ref, Runtime};
use crate::error::{Error, Result, ResultFrom};
use std::{convert::TryFrom, convert::TryInto};
use crate::instructions::exec_method;
use crate::runtime::instance::MemInstance;
use crate::err;

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
            let db: [u8; $s] = self.mem(0)?
                .data[i..i+$s]
                .try_into().wrap("to array")?;
            println!("GET {} BYTES {:?} {}", $s, db, stringify!($t));
            let val = <$t>::from_le_bytes(db);
            Ok(val)
        }
    }
}

macro_rules! set_mem {
    ( $n:ident, $t:ty, $st:ty, $s:expr ) => {
        fn $n(&mut self) -> Result<()> {
            let _a = self.op_u32()?;
            let o = self.op_u32()?;
            let val = self.pop::<$t>()? as $st;
            println!("SETTING {:?}",val);
            let b = self.pop::<u32>()?;
            let bs = val.to_le_bytes();
            let i = (o + b) as usize;
            let m = self.mem(0)?;
            if i+$s > m.data.len() {
                // TODO better error -> trap
                return err!("out of bounds {} {}", i, $s);
            }
            m.data[i..i+$s].clone_from_slice(&bs);
            Ok(())
        }
    }
}

pub trait ExecutionContextActions {
    fn next_byte(&mut self) -> u8;
    fn op_u32(&mut self) -> Result<u32>;
    fn get_local(&mut self, idx: u32) -> Result<Value>;
    fn set_local(&mut self, idx: u32, val: Value) -> Result<()>;

    fn get_global(&mut self, idx: u32) -> Result<Value>;
    fn push_value(&mut self, val: Value) -> Result<()>;
    fn push<T: Into<Value>>(&mut self, val: T) -> Result<()>;
    fn pop_value(&mut self) -> Result<Value>;
    fn pop<T: TryFrom<Value, Error = Error>>(&mut self) -> Result<T>;
    fn call(&mut self, idx: u32) -> Result<()>;
    fn mem(&mut self, idx: u32) -> Result<&mut MemInstance>;

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



impl <'l> ExecutionContextActions for ExecutionContext<'l> {
    fn next_byte(&mut self) -> u8{
        self.body[self.pc]
    }

    fn op_u32(&mut self) -> Result<u32> {
        let result = u32::from_le_bytes(self.body[self.pc..self.pc + 4].try_into().wrap("idx")?);
        self.pc += 4;
        Ok(result)
    }

    fn get_local(&mut self, idx: u32) -> Result<Value> {
        Ok(self.current_frame()?.locals[idx as usize])
    }

    fn set_local(&mut self, idx: u32, val: Value) -> Result<()> {
        self.runtime.stack.mut_activation()?.set_local(idx, val)
    }

    fn mem(&mut self, idx: u32) -> Result<&mut MemInstance> {
        self.runtime.store.mem(idx)
    }

    fn get_global(&mut self, idx: u32) -> Result<Value> {
        let gaddr = idx + self.current_frame()?.module.global_offset;
        self.runtime.store.global(gaddr)
    }

    fn push_value(&mut self, val: Value) -> Result<()> {
        self.runtime.stack.push_value(val);
        Ok(())
    }

    fn push<T: Into<Value>>(&mut self, val: T) -> Result<()> {
        self.push_value(val.into())
    }

    fn pop_value(&mut self) -> Result<Value> {
        self.runtime.stack.pop_value()
    }

    fn pop<T: TryFrom<Value, Error = Error>>(&mut self) -> Result<T> {
        let val = self.pop_value()?;
        println!("POPPED VALUE {:?}", val);
        val.try_into().wrap("pop convert")
    }

    fn call(&mut self, idx: u32) -> Result<()> {
        let faddr = idx + self.current_frame()?.module.func_offset;
        self.runtime.invoke(faddr)
    }
}

impl<'l> ExecutionContext<'l> {
    fn current_frame(&self) -> Result<&ActivationFrame> {
        self.runtime.stack.peek_activation()
    }

    pub fn run(&mut self) -> Result<()> {
        while self.pc < self.body.len() {
            let op = self.body[self.pc];
            println!("HANDLE OP 0x{:x}", op);
            self.pc += 1;
            exec_method(op, self)?;
            println!("FINISHED OP 0x{:x}", op);
        }
        Ok(())
    }
}

/// Implementation of instruction implementation for this runtime.
impl Runtime {
    pub fn enter(&mut self, body: &[u8])-> Result<()> {
        println!("EXECUTING {:x?}", body);
        let mut ic = ExecutionContext {
            runtime: self,
            body,
            pc: 0,
        };
        ic.run()
    }

    pub fn eval_expr(&mut self, body: &[u8]) -> Result<Value> {
        self.enter(body)?;
        self.stack.pop_value()
    }

    pub fn eval_ref_expr(&mut self, body: &[u8]) -> Result<Ref> {
        self.enter(body)?;
        match self.stack.pop_value()? {
            Value::Ref(r) => Ok(r),
            _ => err!("non-ref result for expression")
        }
    }
}
