use {
    super::{
        error::Result,
        format::{SpecParser, SpecTestScript},
    },
    crate::runner::{RunSet, SpecTestRunner},
    std::{fs::File, path::Path},
    wrausmt::{
        format::text::{lex::Tokenizer, parse::Parser},
        loader::Result as LoaderResult,
    },
};

pub fn parse(f: &mut File) -> LoaderResult<SpecTestScript> {
    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let result = parser.parse_spec_test()?;
    Ok(result)
}

pub fn parse_and_run<S: std::fmt::Debug + AsRef<Path>>(path: S, runset: RunSet) -> Result<()> {
    let mut f = std::fs::File::open(&path)?;
    let spectest = parse(&mut f)?;
    let runner = SpecTestRunner::new();
    runner.run_spec_test(spectest, runset)
}
