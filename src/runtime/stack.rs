use super::values::Value;
use super::ModuleInstance;
use crate::error;
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
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#stack
#[derive(Debug, Default)]
pub struct Stack {
    pub value_stack: Vec<Value>,
    pub label_stack: Vec<Label>,
    pub activation_stack: Vec<ActivationFrame>,
}

/// Labels carry an argument arity n and their associated branch target. [Spec][Spec]
///
/// The branch target is expressed syntactically as an instruction sequence. In the
/// implementation, the continuation is represented as the index in the currently
/// executing function that points to the beginning of that instruction sequence.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#labels
#[derive(Debug, Default)]
pub struct Label {
    /// The number of arguments expected by the code for this label.
    pub arity: u32,

    /// the implementation of continuation here is an index into the set of
    /// instructions for the currently executing function.
    pub continuation: u32,
}

/// Activation frames carry the return arity n of the respective function, hold
/// the values of its locals (including arguments) in the order corresponding to
/// their static local indices, and a reference to the function’s own module
/// instance:
#[derive(Debug, Default)]
pub struct ActivationFrame {
    pub arity: u32,
    pub locals: RefCell<Box<[Value]>>,
    pub module: Rc<ModuleInstance>,
}

impl Stack {
    pub fn push_value(&mut self, entry: Value) {
        self.value_stack.push(entry);
    }

    pub fn push_label(&mut self, label: Label) {
        self.label_stack.push(label);
    }
    pub fn push_activation(&mut self, activation: ActivationFrame) {
        self.activation_stack.push(activation);
    }

    pub fn pop_value(&mut self) -> Result<Value> {
        self.value_stack
            .pop()
            .ok_or_else(|| error!("value stack underflow"))
    }
    pub fn pop_label(&mut self) -> Result<Label> {
        self.label_stack
            .pop()
            .ok_or_else(|| error!("label stack underflow"))
    }

    pub fn pop_activation(&mut self) -> Result<ActivationFrame> {
        self.activation_stack
            .pop()
            .ok_or_else(|| error!("activation stack underflow"))
    }

    pub fn peek_activation(&self) -> Result<&ActivationFrame> {
        self.activation_stack
            .last()
            .ok_or_else(|| error!("activation stack underflow"))
    }
}

impl ActivationFrame {
    pub fn new(arity: u32, module: &Rc<ModuleInstance>, locals: Box<[Value]>) -> Self {
        ActivationFrame {
            arity,
            locals: RefCell::new(locals),
            module: module.clone(),
        }
    }
}
