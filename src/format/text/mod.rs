use crate::{
    format::text::parse::error::Result,
    syntax::{Module, Resolved},
};

use {
    self::{
        lex::Tokenizer,
        parse::{
            error::{ParseError, ParseErrorKind},
            Parser,
        },
    },
    std::io::Read,
};

pub mod lex;
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
