pub mod addr;
pub mod data_instance;
pub mod elem_instance;
pub mod export_instance;
pub mod function_instance;
pub mod global_instance;
pub mod mem_instance;
pub mod module_instance;
pub mod table_instance;

pub use {
    data_instance::DataInstance,
    elem_instance::ElemInstance,
    export_instance::{ExportInstance, ExternalVal},
    function_instance::FunctionInstance,
    global_instance::GlobalInstance,
    mem_instance::MemInstance,
    module_instance::ModuleInstance,
    table_instance::TableInstance,
};
