use super::store::Export;
use crate::types::FunctionType;

#[derive(Debug, Default)]
pub struct ModuleInstance {
    pub types: Box<[FunctionType]>,
    pub exports: Box<[Export]>,
    pub func_offset: u32,
}

impl ModuleInstance {
    pub fn resolve(&self, name: &str) -> Option<&Export> {
        let found = self.exports.iter().find(|e| e.name == name);

        found
    }
}
