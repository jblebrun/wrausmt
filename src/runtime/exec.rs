use super::{ActivationFrame, values::Value, values::Ref, Runtime};
use crate::error::{Error, Result, ResultFrom};
use std::{convert::TryFrom, convert::TryInto};
use crate::instructions::exec_method;
use crate::err;

pub struct ExecutionContext<'l> {
    runtime: &'l mut Runtime,
    body: &'l [u8],
    pc: usize,
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
        Ok(self.current_frame()?.locals.borrow()[idx as usize])
    }

    fn set_local(&mut self, idx: u32, val: Value) -> Result<()> {
        self.current_frame()?.locals.borrow_mut()[idx as usize] = val;
        Ok(())
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
        self.pop_value()?.try_into()
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
    #[allow(non_upper_case_globals)]
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
