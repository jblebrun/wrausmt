use super::super::module::Function;
use super::super::types::FunctionType;

/// A function entry in the store.
#[derive(Debug)]
pub struct FunctionInstance<'lt> {
    /// The types of the values returned by the function.
    pub functype: &'lt FunctionType,

    /// The list of instructions in the function.
    pub code: &'lt Function,
}
