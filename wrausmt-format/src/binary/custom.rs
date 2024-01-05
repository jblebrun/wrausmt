use {
    super::{error::Result, BinaryParser, ParserReader},
    crate::{
        binary::{error::ParseResult, read_with_location::Location},
        pctx,
    },
    std::io::Read,
    wrausmt_runtime::syntax,
};

/// Read a custom section, which is interpreted as a simple vec(bytes)
impl<R: ParserReader> BinaryParser<R> {
    pub(in crate::binary) fn read_custom_section(
        &mut self,
        expected_section_size: usize,
    ) -> Result<syntax::CustomField> {
        pctx!(self, "read custom section");
        let name_start = self.location();
        let name = self.read_name()?;
        let name_size = self.location() - name_start;
        let expected_content_size = expected_section_size - name_size;
        let mut content: Vec<u8> = vec![0; expected_content_size];
        self.read_exact(&mut content).result(self)?;
        Ok(syntax::CustomField {
            name,
            content: content.into_boxed_slice(),
        })
    }
}
