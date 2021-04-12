use super::values::{Num, Value};
use super::ModuleInstance;
use crate::error::Result;
use std::cell::RefCell;
use std::rc::Rc;

/// Besides the store, most instructions interact with an implicit stack.
/// [Spec][Spec]
///
///  The stack contains three kinds of entries:
///
///    Values: the operands of instructions.
///    Labels: active structured control instructions that can be targeted by branches.
///    Activations: the call frames of active function calls.
///
/// These entries can occur on the stack in any order during the execution of a
/// program.
///
/// It is possible to model the WebAssembly semantics using separate stacks for
/// operands, control constructs, and calls. However, because the stacks are
/// interdependent, additional book keeping about associated stack heights would
/// be required. For the purpose of this specification, an interleaved
/// representation is simpler.
/// TODO - Consider tracking three separate stacks.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#stack
#[derive(Debug, Default)]
pub struct Stack(Vec<StackEntry>);

/// Stack entries are described by abstract syntax as follows.
#[derive(Debug)]
pub enum StackEntry {
    /// A normal value entry used by operation.
    Value(Value),

    /// A label entry, used for flow control.
    Label { arity: u32, continuation: Rc<[u8]> },

    /// An activation entry, used for function calls.
    Activation { arity: u32, frame: Rc<Frame> },
}

/// Contains the information needed for a function that's executing.
/// local contains the values of params and local variables of the
/// function.
/// module contains a module instance for the module defining the function,
/// which can be used to resolve additional function calls, externals, etc.
#[derive(Debug, Default)]
pub struct Frame {
    pub locals: RefCell<Box<[Value]>>,
    pub module: Rc<ModuleInstance>,
}

impl Frame {
    pub fn new(module: &Rc<ModuleInstance>, locals: Box<[Value]>) -> Frame {
        Frame {
            locals: RefCell::new(locals),
            module: module.clone(),
        }
    }
}

impl Stack {
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
            _ => Err("Stack assertion".into()),
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
    };
}

intostack! { u32, v, Value::Num(Num::I32(v))}
intostack! { u64, v, Value::Num(Num::I64(v))}
intostack! { f32, v, Value::Num(Num::F32(v))}
intostack! { f64, v, Value::Num(Num::F64(v))}
intostack! { Value, v, v }
intostack! { &Value, v, *v }
