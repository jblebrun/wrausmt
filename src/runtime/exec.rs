use super::Runtime;
use std::convert::TryInto;
use super::super::instructions::*;
use super::super::error::*;
use super::stack::Frame;
use std::rc::Rc;

struct InvocationContext<'l> {
    runtime: &'l mut Runtime,
    body: &'l [u8],
    pc: usize
}

impl<'l> InvocationContext<'l> {
    fn next_u32(&mut self) -> Result<u32> {
        let result = u32::from_le_bytes(self.body[self.pc..self.pc+4].try_into().wrap("idx")?);
        self.pc += 4;
        Ok(result)
    }

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
                    self.runtime.stack.push(idx.into());
                },
                LocalGet => {
                    let idx = self.next_u32()?;
                    let val = self.current_frame()?.locals.borrow()[idx as usize];
                    println!("GOT LOCAL {:?}", val);
                    self.runtime.stack.push(val.into());
                },
                LocalSet => {
                    let idx = self.next_u32()?;
                    let val = self.runtime.stack.pop_value()?;
                    self.current_frame()?.locals.borrow_mut()[idx as usize] = val;
                }
                I32Const => {
                    let val = self.next_u32()?;
                    self.runtime.stack.push(val.into());
                },
                I32Add => {
                    let a = self.runtime.stack.pop_i32()?;
                    let b = self.runtime.stack.pop_i32()?;
                    self.runtime.stack.push((a+b).into());
                },
                I32Sub => {
                    let a = self.runtime.stack.pop_i32()?;
                    let b = self.runtime.stack.pop_i32()?;
                    self.runtime.stack.push((a-b).into());
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
