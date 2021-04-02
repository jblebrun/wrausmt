use super::super::instructions::Inst;
use std::rc::Rc;
use super::ModuleInstance;

/// Contains the information needed for a function that's executing.
/// local contains the values of params and local variables of the
/// function.
/// module contains a module instance for the module defining the function,
/// which can be used to resolve additional function calls, externals, etc.
#[derive(Debug)]
pub struct Frame {
    pub locals: Box<[u64]>,
    pub module: Rc<ModuleInstance>
}

/// A single entry on the runtime stack.
#[derive(Debug)]
pub enum StackEntry {
    /// A normal value entry used by operation.
    Value(u64),

    /// A label entry, used for flow control.
    Label { arity: u32, continuation: Rc<[Inst]> },
    
    /// An activation entry, used for function calls.
    Activation { arity: u32, frame: Rc<Frame> }
}


/// The runtime stack for the WASM machine. It contains
/// the values used as operands to instructions, context for
/// active functions, and labels for flow control operations.
#[derive(Debug)]
pub struct Stack(Vec<StackEntry>);


impl Stack {
    pub fn new() -> Stack { Stack(vec![]) }

    pub fn push(&mut self, entry: StackEntry) {
        self.0.push(entry);
    }

    pub fn pop(&mut self) -> Option<StackEntry> {
        self.0.pop()
    }

    pub fn pop_value(&mut self) -> u64 {
        // To investigate - in validated mode,
        // is it possible to remove all checks here,
        // and simply unwrap the popped value, assuming
        // it's Some(Value(_))?
        match self.0.pop() {
            Some(StackEntry::Value(val)) => val,
            _ => panic!("Stack assertion")
        }
    }
}
