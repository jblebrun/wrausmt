use std::rc::Rc;
use super::super::types::*;
use super::ModuleInstance;
use super::super::error::Result;
use std::cell::RefCell;

/// Contains the information needed for a function that's executing.
/// local contains the values of params and local variables of the
/// function.
/// module contains a module instance for the module defining the function,
/// which can be used to resolve additional function calls, externals, etc.
#[derive(Debug)]
pub struct Frame {
    pub locals: RefCell<Box<[Value]>>,
    pub module: Rc<ModuleInstance>
}

impl Frame {
    pub fn dummy() -> Frame {
        Frame {
            locals: RefCell::new(Box::new([])), 
            module: Rc::new(ModuleInstance::empty())
        }
    }

    pub fn new(
        module: &Rc<ModuleInstance>,
        locals: Box<[Value]>
    ) -> Frame {
        Frame {
            locals: RefCell::new(locals),
            module: module.clone(),
        }
    }
}
/// A single entry on the runtime stack.
#[derive(Debug)]
pub enum StackEntry {
    /// A normal value entry used by operation.
    Value(Value),

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

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    pub fn new() -> Stack { Stack(vec![]) }

    pub fn push(&mut self, entry: StackEntry) {
        self.0.push(entry);
    }

    pub fn pop(&mut self) -> Option<StackEntry> {
        self.0.pop()
    }

    pub fn pop_value(&mut self) -> Result<Value> {
        // To investigate - in validated mode,
        // is it possible to remove all checks here,
        // and simply unwrap the popped value, assuming
        // it's Some(Value(_))?
        match self.0.pop() {
            Some(StackEntry::Value(val)) => Ok(val),
            _ => Err("Stack assertion".into())
        }
    }
}

macro_rules! intostack {
    ( $ty:ty, $id:ident, $res:expr ) => {
        impl From<$ty> for StackEntry {
            fn from($id: $ty) -> StackEntry {
                StackEntry::Value($res)
            }
        }
    }
}

intostack! { u32, v, Value::Num(Num::I32(v))}
intostack! { u64, v, Value::Num(Num::I64(v))}
intostack! { f32, v, Value::Num(Num::F32(v))}
intostack! { f64, v, Value::Num(Num::F64(v))}
intostack! { Value, v, v }
intostack! { &Value, v, *v }


