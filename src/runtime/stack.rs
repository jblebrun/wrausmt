use super::super::instructions::Inst;
use std::rc::Rc;
use super::ModuleInstance;

#[derive(Debug)]
pub struct Frame {
    pub arity: u32,
    pub locals: Box<[u64]>,
    pub module: Rc<ModuleInstance>
}

#[derive(Debug)]
pub enum StackEntry {
    Value(u64),
    Label { arity: u32, continuation: Rc<[Inst]> },
    Activation { arity: u32, frame: Rc<Frame> }
}


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
