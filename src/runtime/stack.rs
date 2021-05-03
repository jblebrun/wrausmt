use super::{instance::FunctionInstance, store::addr, values::Value};
use super::ModuleInstance;
use crate::error;
use crate::error::Result;
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
    value_stack: Vec<Value>,
    label_stack: Vec<Label>,
    activation_stack: Vec<ActivationFrame>,
}

/// Labels carry an argument arity n and their associated branch target. [Spec][Spec]
///
/// The branch target is expressed syntactically as an instruction sequence. In the
/// implementation, the continuation is represented as the index in the currently
/// executing function that points to the beginning of that instruction sequence.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#labels
#[derive(Debug, PartialEq, Default)]
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
struct ActivationFrame {
    pub arity: u32,
    /// The value stack also contains the locals for the current frame.
    /// This value contains the index into the stack for the frame.
    pub local_start: usize,
    pub module: Rc<ModuleInstance>,
}

impl Stack {
    pub fn push_value(&mut self, entry: Value) {
        self.value_stack.push(entry);
    }

    pub fn push_label(&mut self, label: Label) {
        self.label_stack.push(label);
    }

    pub fn push_activation(&mut self, funcinst: &FunctionInstance) -> Result<()> {
        let frame_start = self.value_stack.len() - funcinst.functype.params.len();
        // 8. Let val0* be the list of zero values (other locals). 
        for localtype in funcinst.code.locals.iter() {
            self.push_value(localtype.default());
        }
        println!("FRAME START: {}", frame_start);
        self.activation_stack.push(ActivationFrame {
                arity: funcinst.functype.result.len() as u32,
                local_start: frame_start,
                module: funcinst.module_instance()?,
        });
        Ok(())
    }

    pub fn push_dummy_activation(&mut self, modinst: Rc<ModuleInstance>) -> Result<()> {
        self.activation_stack.push(ActivationFrame{
                arity: 0,
                local_start: self.value_stack.len(),
                module: modinst,
        });
        Ok(())
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

    pub fn pop_activation(&mut self) -> Result<()> {
        let frame = self.activation_stack
            .pop()
            .ok_or_else(|| error!("activation stack underflow"))?;

        // Move the results to the new top of the stack.
        for i in 0..frame.arity as usize {
            self.value_stack[frame.local_start+i] = 
                self.value_stack[self.value_stack.len()-i-1];
        }

        // Pop the rest of the frame.
        // We originally had params + locals. 
        // At the end, there were also results. We moved the results to the bottom.
        // Now we just need to truncate away the params/locals.
        let truncated_size = frame.local_start + frame.arity as usize;
        self.value_stack.truncate(truncated_size);
        Ok(())
    }

    pub fn activation_depth(&self) -> usize { self.activation_stack.len() }

    pub fn peek_label(&self) -> Result<&Label> {
        self.label_stack.last()
            .ok_or_else(|| error!("label stack underflow"))
    }

    fn peek_activation(&self) -> Result<&ActivationFrame> {
        self.activation_stack
            .last()
            .ok_or_else(|| error!("activation stack underflow"))
    }

    // Get the local at the provided index for the current activation frame.
    pub fn get_local(&self, idx: u32) -> Result<Value> {
        let localidx = self.peek_activation()?.local_start;
        Ok(self.value_stack[localidx + idx as usize])
    }

    pub fn set_local(&mut self, idx: u32, val: Value) -> Result<()> {
        let localidx = self.peek_activation()?.local_start;
        self.value_stack[localidx + idx as usize] = val;
        Ok(())
    }

    // Get the function address for the provided index in the current activation.
    pub fn get_function_addr(&self, idx: u32) -> Result<addr::FuncAddr> {
        Ok(self.peek_activation()?.module.func_offset + idx)
    }
    
    // Get the global address for the provided index in the current activation.
    pub fn get_global_addr(&self, idx: u32) -> Result<addr::GlobalAddr> {
        Ok(self.peek_activation()?.module.global_offset + idx)
    }

    pub fn get_label(&self, idx: u32) -> Result<&Label> {
        let fromend = self.label_stack.len() as u32 - 1 - idx;
        Ok(&self.label_stack[fromend as usize])
    }
}
