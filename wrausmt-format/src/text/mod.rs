use {
    self::parse::Parser,
    super::text::parse::error::Result,
    std::io::Read,
    wrausmt_runtime::syntax::{Module, Resolved, UncompiledExpr},
};

pub mod lex;
pub mod location;
pub mod num;
pub mod token;

pub mod macros;
pub mod module_builder;
pub mod parse;
pub mod resolve;
pub mod string;

pub fn parse_wast_data(
    reader: &mut impl Read,
) -> Result<Module<Resolved, UncompiledExpr<Resolved>>> {
    let mut parser = Parser::new(reader);
    parser.parse_full_module()
}
