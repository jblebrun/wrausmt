use super::Runtime;
use std::convert::TryInto;
use super::super::instructions::*;
use super::super::error::*;
use super::stack::Frame;
use std::rc::Rc;
use super::super::types::*;
use std::convert::TryFrom;

struct InvocationContext<'l> {
    runtime: &'l mut Runtime,
    body: &'l [u8],
    pc: usize
}

trait ActivationContext {
    fn next_byte(&mut self) -> u8;
    fn next_u32(&mut self) -> Result<u32>;
    fn get_local(&mut self, idx: u32) -> Result<Value>;
    fn set_local(&mut self, idx: u32, val: Value) -> Result<()>;
    fn push_value(&mut self, val: Value) -> Result<()>;
    fn push<T : Into<Value>>(&mut self, val: T) -> Result<()>;
    fn pop_value(&mut self) -> Result<Value>;
    fn pop<T : TryFrom<Value, Error=Error>>(&mut self) -> Result<T>;
}

impl <'l> ActivationContext for InvocationContext<'l> {
    fn next_byte(&mut self) -> u8{
        self.body[self.pc]
    }

    fn next_u32(&mut self) -> Result<u32> {
        let result = u32::from_le_bytes(self.body[self.pc..self.pc+4].try_into().wrap("idx")?);
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
        self.runtime.stack.push(val.into());
        Ok(())
    }
    
    fn push<T : Into<Value>>(&mut self, val: T) -> Result<()> {
        self.push_value(val.into())
    }

    fn pop_value(&mut self) -> Result<Value> {
        self.runtime.stack.pop_value()
    }

    fn pop<T : TryFrom<Value, Error=Error>>(&mut self) -> Result<T> {
        self.pop_value()?.try_into()
    }
}

impl<'l> InvocationContext<'l> {


    fn current_frame(&self) -> Result<Rc<Frame>> {
        match &self.runtime.current_frame {
            Some(frame) => Ok(frame.clone()),
            _ => Err(Error::new("no current frame".to_string()))
        }
    }

    #[allow(non_upper_case_globals)]
    pub fn run(&mut self) -> Result<()> {
        while self.pc < self.body.len() {
            let op = self.body[self.pc];
            println!("HANDLE OP {}", op);
            self.pc += 1;
            match op {
                Return => {
                    return Ok(())
                },
                GlobalGet => {
                    let idx = self.next_u32()?;
                    // TODO - actual find a global
                    self.push(idx)?;
                },
                LocalGet => {
                    let idx = self.next_u32()?;
                    let val = self.get_local(idx)?;
                    self.push_value(val)?;
                },
                LocalSet => {
                    let idx = self.next_u32()?;
                    let val = self.pop_value()?;
                    self.set_local(idx, val)?;
                }
                I32Const => {
                    let val = self.next_u32()?;
                    self.push_value(val.into())?;
                },
                I32Add => {
                    let a = self.pop::<u32>()?;
                    let b = self.pop::<u32>()?;
                    self.push(a+b)?;
                },
                I32Sub => {
                    let a = self.pop::<u32>()?;
                    let b = self.pop::<u32>()?;
                    self.push(a-b)?;
                },
                I32Load => {
                    let _align = self.next_u32()?;
                    let _offset= self.next_u32()?;
                    // TODO - actual load memory
                },
                I32Store => {
                    let _align = self.next_u32()?;
                    let _offset= self.next_u32()?;
                    // TODO - actual store memory
                },
                End => {
                    return Ok(())
                }
                _ => panic!("not yet for {:x}", op)
            }
        }
        Ok(())
    }
}

/// Implementation of instruction implementation for this runtime.
impl Runtime {
    pub fn enter(&mut self, body: &[u8]) -> Result<()> { 
        println!("EXECUTING {:x?}", body);
        let mut ic = InvocationContext {
            runtime: self,
            body,
            pc: 0
        };
        ic.run()
    }
}
