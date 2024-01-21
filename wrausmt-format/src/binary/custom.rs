use {
    super::{error::Result, BinaryParser, ParserReader},
    crate::{
        binary::{
            error::{BinaryParseErrorKind, ParseResult},
            leb128::ReadLeb128,
            read_with_location::Locate,
        },
        pctx,
    },
    std::io::Read,
    wrausmt_runtime::syntax,
};

/// Read a custom section, which is interpreted as a simple vec(bytes)
impl<R: ParserReader> BinaryParser<R> {
    pub(in crate::binary) fn read_custom_section(&mut self) -> Result<syntax::CustomField> {
        pctx!(self, "read custom section");
        let expected_size = self.read_u32_leb_128().result(self)?;

        if expected_size == 0 {
            return Err(self.err(BinaryParseErrorKind::UnexpectedEnd));
        }
        let name_start = self.location().pos;
        let name = self.read_name()?;
        let name_size = self.location().pos - name_start;
        let expected_content_size = expected_size - name_size;
        let mut content: Vec<u8> = vec![0; expected_content_size as usize];
        self.read_exact(&mut content).result(self)?;
        Ok(syntax::CustomField {
            name,
            content: content.into_boxed_slice(),
        })
    }
}
