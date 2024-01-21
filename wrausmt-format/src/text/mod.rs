use {
    self::{
        module_builder::ModuleIdentifiers,
        parse::Parser,
        resolve::{ResolutionContext, Resolve},
    },
    super::text::parse::error::Result,
    std::io::Read,
    wrausmt_runtime::syntax::{Module, Resolved, UncompiledExpr, Unresolved},
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
) -> Result<Module<Resolved, UncompiledExpr<Resolved>>> {
    let mut parser = Parser::new(reader);
    parser.parse_full_module()
}

/// This is for internal use for very simple inline expressions that get
/// inserted during compilation. It's very quick and dirty, and is hard to debug
/// if something goes wrong.
/// If parsing into a resolved instruction, no named indices can be used, or
/// anything else that gets worked out at resolution time.
/// This is suitable for constant expressions.
pub(crate) fn parse_text_resolved_instructions(data: &str) -> UncompiledExpr<Resolved> {
    let expr = parse_text_unresolved_instructions(data);
    let mut types = Vec::new();
    let module_identifiers = ModuleIdentifiers::default();
    let mut rc = ResolutionContext {
        types:        &mut types,
        modulescope:  &module_identifiers,
        localindices: Vec::new(),
        labelindices: Vec::new(),
    };
    expr.resolve(&mut rc).unwrap()
}

pub(crate) fn parse_text_unresolved_instructions(data: &str) -> UncompiledExpr<Unresolved> {
    let mut parser = Parser::new(data.as_bytes());
    parser.assure_started().unwrap();
    let instr = parser.parse_instructions().unwrap();
    UncompiledExpr { instr }
}
