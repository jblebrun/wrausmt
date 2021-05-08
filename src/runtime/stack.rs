use super::ModuleInstance;
use super::{instance::FunctionInstance, store::addr, values::Value};
use crate::{err, error::Result};
use crate::{
    error,
    logger::{Logger, PrintLogger},
};
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
    activation_stack: Vec<ActivationFrame>,
    logger: PrintLogger,
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

    /// The location of the value stack when the label is pushed; block return values
    /// will be moved here when exiting a block.
    pub return_spot: usize,
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
    label_stack: Vec<Label>,
}

impl Stack {
    pub fn push_value(&mut self, entry: Value) {
        self.value_stack.push(entry);
        self.logger.log("VALSTACK", || format!("PUSH {:?}", entry));
        self.logger
            .log("DUMPVALSTACK", || format!("{:?}", self.value_stack));
    }

    fn label_stack(&self) -> Result<&Vec<Label>> {
        Ok(self.peek_activation()?.label_stack.as_ref())
    }

    fn label_stack_mut(&mut self) -> Result<&mut Vec<Label>> {
        Ok(self.peek_activation_mut()?.label_stack.as_mut())
    }

    pub fn push_label(
        &mut self,
        param_arity: u32,
        result_arity: u32,
        continuation: u32,
    ) -> Result<()> {
        let label = Label {
            arity: result_arity,
            continuation,
            return_spot: self.value_stack.len() - param_arity as usize,
        };
        self.logger.log("LABSTACK", || format!("PUSH {:?}", label));
        self.label_stack_mut()?.push(label);
        self.logger
            .log("DUMPLABSTACK", || format!("{:?}", self.label_stack()));
        Ok(())
    }

    pub fn push_activation(&mut self, funcinst: &FunctionInstance) -> Result<()> {
        if self.activation_stack.len() > 256 {
            return err!("stack overflow");
        }

        let frame_start = self.value_stack.len() - funcinst.functype.params.len();
        // 8. Let val0* be the list of zero values (other locals).
        for localtype in funcinst.locals.iter() {
            self.push_value(localtype.default());
        }

        let arity = funcinst.functype.result.len() as u32;

        self.activation_stack.push(ActivationFrame {
            arity,
            local_start: frame_start,
            module: funcinst.module_instance()?,
            label_stack: vec![],
        });
        self.logger.log("ACTIVATE", || {
            format!(
                "arity {} local_start {} stack size {}",
                arity,
                frame_start,
                self.activation_stack.len()
            )
        });
        Ok(())
    }

    pub fn push_dummy_activation(&mut self, modinst: Rc<ModuleInstance>) -> Result<()> {
        self.activation_stack.push(ActivationFrame {
            arity: 0,
            local_start: self.value_stack.len(),
            module: modinst,
            label_stack: vec![],
        });
        Ok(())
    }

    pub fn pop_value(&mut self) -> Result<Value> {
        self.value_stack
            .pop()
            .ok_or_else(|| error!("value stack underflow"))
    }

    pub fn pop_label(&mut self) -> Result<Label> {
        self.label_stack_mut()?
            .pop()
            .ok_or_else(|| error!("label stack underflow"))
        // For non-break block exists, the stack is assumed to be proper,
        // no adjustment needed.
    }

    fn break_label(&mut self) -> Result<Label> {
        let label = self.pop_label()?;
        self.move_return_values(label.arity, label.return_spot)?;
        Ok(label)
    }

    pub fn break_to_label(&mut self, labelidx: u32) -> Result<Label> {
        let mut label: Option<Label> = None;
        for _ in 0..=labelidx {
            label = Some(self.break_label()?);
        }
        let label = label.ok_or("failed to finish break")?;
        Ok(label)
    }

    // Handle adjusting return values to a new stack top for breaks and returns.
    fn move_return_values(&mut self, arity: u32, newtop: usize) -> Result<()> {
        self.logger.log("STACK", || {
            format!("MOVING RETURN VALUES FOR {} {}", arity, newtop)
        });
        self.logger
            .log("DUMPSTACK", || format!("STACK {:?}", self.value_stack));
        let result_start = self.value_stack.len() - (arity as usize);
        self.value_stack.copy_within(result_start.., newtop);
        self.logger.log("DUMPSTACK", || {
            format!("AFTER MOVE STACK {:?}", self.value_stack)
        });

        let truncated_size = newtop + arity as usize;
        self.value_stack.truncate(truncated_size);

        Ok(())
    }

    pub fn pop_activation(&mut self) -> Result<()> {
        let frame = self
            .activation_stack
            .pop()
            .ok_or_else(|| error!("activation stack underflow"))?;

        self.move_return_values(frame.arity, frame.local_start)?;
        Ok(())
    }

    pub fn activation_depth(&self) -> usize {
        self.activation_stack.len()
    }

    pub fn peek_label(&self) -> Result<&Label> {
        self.label_stack()?
            .last()
            .ok_or_else(|| error!("label stack underflow"))
    }

    fn peek_activation(&self) -> Result<&ActivationFrame> {
        self.activation_stack
            .last()
            .ok_or_else(|| error!("activation stack underflow"))
    }

    fn peek_activation_mut(&mut self) -> Result<&mut ActivationFrame> {
        self.activation_stack
            .last_mut()
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

    // Get the function address for the provided index in the current activation.
    pub fn get_mem_addr(&self, idx: u32) -> Result<addr::FuncAddr> {
        Ok(self.peek_activation()?.module.mem_offset + idx)
    }

    // Get the global address for the provided index in the current activation.
    pub fn get_global_addr(&self, idx: u32) -> Result<addr::GlobalAddr> {
        Ok(self.peek_activation()?.module.global_offset + idx)
    }

    // Get the function address for the provided index in the current activation.
    pub fn get_table_addr(&self, idx: u32) -> Result<addr::TableAddr> {
        Ok(self.peek_activation()?.module.table_offset + idx)
    }

    // Get the function address for the provided index in the current activation.
    pub fn get_elem_addr(&self, idx: u32) -> Result<addr::ElemAddr> {
        Ok(self.peek_activation()?.module.elem_offset + idx)
    }

    pub fn get_label(&self, idx: u32) -> Result<&Label> {
        let fromend = self.label_stack()?.len() as u32 - 1 - idx;
        Ok(&self.label_stack()?[fromend as usize])
    }
}
