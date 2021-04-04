use super::Runtime;
use super::stack::StackEntry::*;
use std::convert::TryInto;
use super::super::instructions::*;
use super::super::error::*;

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

    #[allow(non_upper_case_globals)]
    pub fn run(&mut self) -> Result<()> {
        while self.pc < self.body.len() {
            let op = self.body[self.pc];
            println!("HANDLE OP {}", op);
            self.pc += 1;
            match op {
                LocalGet => {
                    let idx = self.next_u32()?;
                    let val = match &self.runtime.current_frame {
                        Some(frame) => frame.locals[idx as usize],
                        _ => panic!("no current frame")
                    };
                    self.runtime.stack.push(Value(val));
                },
                I32Const => {
                    let val = self.next_u32()?;
                    self.runtime.stack.push(Value(val as u64));
                },
                I32Add => {
                    let a = self.runtime.stack.pop_value();
                    let b = self.runtime.stack.pop_value();
                    self.runtime.stack.push(Value(a+b));
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
    pub fn invoke(&mut self, body: &[u8]) -> Result<()> { 
        println!("EXECUTING {:x?}", body);
        let mut ic = InvocationContext {
            runtime: self,
            body,
            pc: 0
        };
        ic.run()
    }
}
