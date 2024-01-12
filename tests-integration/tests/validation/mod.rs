use {
    tests::spec::{
        error::Result,
        loader::parse_and_run,
        runner::{RunConfig, RunSet},
    },
    wrausmt_format::ValidationMode,
};

const RUN_CONFIG: RunConfig = RunConfig {
    runset:             RunSet::All,
    validation_mode:    ValidationMode::Fail,
    failures_to_ignore: &[],
};

#[test]
fn basic_passing_validation() -> Result<()> {
    parse_and_run("tests/validation/data/validation.wat", RUN_CONFIG)
}

#[test]
fn basic_failing_validation() -> Result<()> {
    parse_and_run("tests/validation/data/fail_validation.wat", RUN_CONFIG)
}
