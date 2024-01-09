use {
    super::{error::Result, BinaryParser, ParserReader},
    crate::{
        binary::{
            error::{BinaryParseErrorKind, EofAsKind, ParseResult},
            leb128::ReadLeb128,
        },
        pctx,
    },
    std::io::Read,
    wrausmt_runtime::syntax,
};

impl<R: ParserReader> BinaryParser<R> {
    /// Read the next non-custom section ID in the binary module and return it.
    /// Any custom sections encountered beforehand will be returned.
    ///
    /// [Spec]: https://webassembly.github.io/spec/core/binary/modules.html#sections
    pub(in crate::binary) fn read_next_section_id(
        &mut self,
        customs: &mut Vec<syntax::CustomField>,
    ) -> Result<Option<u8>> {
        pctx!(self, "read section");
        loop {
            match (&mut self.reader).bytes().next() {
                Some(Ok(0)) => {
                    customs.push(
                        self.read_custom_section()
                            .eof_as_kind(BinaryParseErrorKind::UnexpectedEnd)?,
                    );
                }
                Some(Ok(v @ 1..=12)) => {
                    return Ok(Some(v));
                }
                Some(Ok(v)) => Err(self.err(BinaryParseErrorKind::MalformedSectionId(v)))?,
                Some(Err(e)) => Err(e).result(self)?,
                None => return Ok(None),
            };
        }
    }

    pub(in crate::binary) fn read_section<S>(
        &mut self,
        parsefn: impl Fn(&mut Self) -> Result<S>,
    ) -> Result<S> {
        let expected_size = self.read_u32_leb_128().result(self)? as usize;
        let (section, amount_read) = self
            .count_reads(parsefn)
            .eof_as_kind(BinaryParseErrorKind::UnxpectedEndOfSectionOrFunction)?;
        println!("EXPECTED {} READ {}", expected_size, amount_read);
        match amount_read {
            cnt if cnt < expected_size => Err(self.err(BinaryParseErrorKind::SectionTooShort)),
            cnt if cnt > expected_size => Err(self.err(BinaryParseErrorKind::SectionTooLong)),
            _ => Ok(section),
        }
    }
}
