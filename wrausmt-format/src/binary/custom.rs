use {
    super::{
        error::{Result, WithContext},
        BinaryParser,
    },
    std::io::Read,
};

/// Read a custom section, which is interpreted as a simple vec(bytes)
impl<R: Read> BinaryParser<R> {
    pub(in crate::binary) fn read_custom_section(&mut self) -> Result<Box<[u8]>> {
        let mut section: Vec<u8> = vec![];
        self.read_to_end(&mut section)
            .ctx("reading custom content")?;
        Ok(section.into_boxed_slice())
    }
}
