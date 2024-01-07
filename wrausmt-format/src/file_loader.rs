use {
    crate::loader::{Loader, Result},
    std::{
        fs::File,
        io::{Read, Seek, SeekFrom},
        rc::Rc,
    },
    wrausmt_runtime::{
        runtime::{instance::ModuleInstance, Runtime},
        validation::ValidationMode,
    },
};

pub trait FileLoader: Loader {
    /// Load a WASM or WAST file. The loader will look for the magic binary
    /// bytes at the start. If those are not found, it will try loading the file
    /// as a text-format file.
    fn load_file(
        &mut self,
        filename: &str,
        validation_mode: ValidationMode,
    ) -> Result<Rc<ModuleInstance>> {
        let mut file = File::open(filename)?;
        let mut magic: [u8; 4] = [0u8; 4];
        file.read_exact(&mut magic)?;
        file.seek(SeekFrom::Start(0))?;
        if &magic == b"\0asm" {
            println!("Magic header exists... Attemptin load as WASM binary format.");
            self.load_wasm_data(&mut file, validation_mode)
        } else {
            println!("Magic header doesn't exist... Attempting load as WASM text format.");
            self.load_wast_data(&mut file, validation_mode)
        }
    }
}

impl FileLoader for Runtime {}
