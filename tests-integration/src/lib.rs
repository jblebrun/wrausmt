use {
    std::rc::Rc,
    wrausmt_format::{
        file_loader::FileLoader,
        loader::{Loader, Result},
    },
    wrausmt_runtime::{
        runtime::{instance::ModuleInstance, Runtime},
        validation::ValidationMode,
    },
};

pub trait TestLoader: Loader {
    fn load_test_file(&mut self, filename: &str) -> Result<Rc<ModuleInstance>>;
    fn load_and_register_wast_file(&mut self, filename: &str, modname: &str) -> Result<()>;
    fn load_env(&mut self) -> Result<()> {
        self.load_and_register_wast_file("data/env.wasm", "env")
    }
}

impl TestLoader for Runtime {
    fn load_test_file(&mut self, filename: &str) -> Result<Rc<ModuleInstance>> {
        self.load_file(filename, ValidationMode::Warn)
    }

    fn load_and_register_wast_file(&mut self, filename: &str, modname: &str) -> Result<()> {
        let module = self.load_file(filename, ValidationMode::Warn)?;
        self.register(modname, module);
        Ok(())
    }
}
