use {
    self::{
        lex::Tokenizer,
        parse::{
            error::{ParseError, ParseErrorKind},
            Parser,
        },
    },
    super::text::parse::error::Result,
    std::io::Read,
    wrausmt::syntax::{Module, Resolved},
};

pub mod lex;
pub mod num;
pub mod token;

pub mod macros;
pub mod module_builder;
pub mod parse;
pub mod resolve;
pub mod string;

pub fn parse_wast_data(reader: &mut impl Read) -> Result<Module<Resolved>> {
    let tokenizer = Tokenizer::new(reader)
        .map_err(|e| ParseError::new_nocontext(ParseErrorKind::LexError(e)))?;
    let mut parser = Parser::new(tokenizer)?;
    parser.parse_full_module()
}
