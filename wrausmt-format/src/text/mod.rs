use {
    self::parse::Parser,
    super::text::parse::error::Result,
    std::io::Read,
    wrausmt_runtime::syntax::{Module, Resolved, UncompiledExpr, Unresolved, Unvalidated},
};

pub mod lex;
pub mod num;
pub mod token;

pub mod macros;
pub mod module_builder;
pub mod parse;
pub mod resolve;
pub mod string;

pub fn parse_wast_data(
    reader: &mut impl Read,
) -> Result<Module<Resolved, Unvalidated, UncompiledExpr<Resolved>>> {
    let mut parser = Parser::new(reader);
    parser.parse_full_module()
}

pub(crate) fn parse_text_unresolved_instructions(data: &str) -> UncompiledExpr<Unresolved> {
    let mut parser = Parser::new(data.as_bytes());
    parser.assure_started().unwrap();
    let instr = parser.parse_instructions().unwrap();
    UncompiledExpr { instr }
}
