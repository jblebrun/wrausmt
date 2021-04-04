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

impl Frame {
    pub fn dummy() -> Frame {
        Frame {
            locals: Box::new([]), 
            module: Rc::new(ModuleInstance::empty())
        }
    }

    pub fn new(
        module: &Rc<ModuleInstance>,
        locals: Box<[u64]>
    ) -> Frame {
        Frame {
            locals,
            module: module.clone(),
        }
    }
}
/// A single entry on the runtime stack.
#[derive(Debug)]
pub enum StackEntry {
    /// A normal value entry used by operation.
    Value(u64),

    /// A label entry, used for flow control.
    Label { arity: u32, continuation: Rc<[u8]> },
    
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

impl From<&u64> for StackEntry {
    fn from(val: &u64) -> StackEntry {
        StackEntry::Value(*val)
    }
}

impl From<u64> for StackEntry {
    fn from(val: u64) -> StackEntry {
        StackEntry::Value(val)
    }
}

impl From<u32> for StackEntry {
    fn from(val: u32) -> StackEntry {
        StackEntry::Value(val as u64)
    }
}



