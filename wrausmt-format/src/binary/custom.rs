use {
    super::{error::Result, BinaryParser},
    crate::{binary::error::ParseResult, pctx},
    std::io::Read,
    wrausmt_runtime::syntax,
};

/// Read a custom section, which is interpreted as a simple vec(bytes)
impl<R: Read> BinaryParser<R> {
    pub(in crate::binary) fn read_custom_section(&mut self) -> Result<syntax::CustomField> {
        pctx!(self, "read custom section");
        let name = self.read_name()?;
        let mut section: Vec<u8> = vec![];
        self.read_to_end(&mut section).result(self)?;
        Ok(syntax::CustomField {
            name,
            content: section.into_boxed_slice(),
        })
    }
}
