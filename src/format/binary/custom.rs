use crate::error::{Result, ResultFrom};
use std::io::Read;

/// Read a custom section, which is interpreted as a simple vec(bytes)
pub trait ReadCustom: Read {
    fn read_custom_section(&mut self) -> Result<Box<[u8]>> {
        let mut section: Vec<u8> = vec![];
        self.read_to_end(&mut section)
            .wrap("reading custom content")?;
        println!("CUSTOM: {:?}", section);
        Ok(section.into_boxed_slice())
    }
}

impl<I: Read> ReadCustom for I {}
