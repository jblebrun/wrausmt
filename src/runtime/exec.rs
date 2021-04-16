use super::{stack::ActivationFrame, values::Value, Runtime};
use crate::error::{Error, Result, ResultFrom};
use std::{convert::TryFrom, convert::TryInto};
use crate::instructions::exec_method;

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
    fn push_value(&mut self, val: Value) -> Result<()>;
    fn push<T: Into<Value>>(&mut self, val: T) -> Result<()>;
    fn pop_value(&mut self) -> Result<Value>;
    fn pop<T: TryFrom<Value, Error = Error>>(&mut self) -> Result<T>;
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
        }
        Ok(())
    }
}

/// Implementation of instruction implementation for this runtime.
impl Runtime {
    pub fn enter(&mut self, body: &[u8]) -> Result<()> {
        println!("EXECUTING {:x?}", body);
        let mut ic = ExecutionContext {
            runtime: self,
            body,
            pc: 0,
        };
        ic.run()
    }
}
