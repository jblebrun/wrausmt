use super::super::instructions::Inst;
use super::super::instructions::Inst::*;
use super::Runtime;
use super::stack::StackEntry::*;

/// Implementation of instruction implementation for this runtime.
impl Inst {
    pub fn execute(&self, runtime: &mut Runtime) {
        match self {
            // Get the local variable for the specified index,
            // and place it on the stack.
            LocalGet(idx) => {
                let val = match &runtime.current_frame {
                    Some(frame) => frame.locals[*idx as usize],
                    _ => panic!("no current frame")
                };
                runtime.stack.push(Value(val));
            },

            // Place the provided constant value on the stack.
            I32_Const(val) => {
                runtime.stack.push(Value(*val as u64));
            },

            // Pop the top two values from the stack.
            // Push the result of adding those two values.
            Add32 => {
                let a = runtime.stack.pop_value();
                let b = runtime.stack.pop_value();
                runtime.stack.push(Value(a+b));
            }
            _ => panic!("not yet")
        }
    }
}
