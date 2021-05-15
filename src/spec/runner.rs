use std::rc::Rc;

use super::{
    format::{Action, ActionResult, NumPat, SpecTestScript},
    spectest_module::make_spectest_module,
};
use crate::{
    err,
    error::{Error, ErrorFrom, Result, ResultFrom},
    format::{binary::parse, Location},
    logger::{Logger, PrintLogger},
    runtime::{
        instance::ModuleInstance,
        values::{Num, Ref, Value},
        Runtime,
    },
    spec::format::{Assertion, Cmd, Module},
};

#[macro_export]
macro_rules! runset_specific {
    ( $( $n:expr ),* ) => {
        RunSet::Specific(vec![
            $(
                $n.to_owned(),
            )*
        ])
    }
}

#[macro_export]
macro_rules! runset_exclude {
    ( $( $n:expr ),* ) => {
        RunSet::Exclude(vec![
            $(
                $n.to_owned(),
            )*
        ])
    }
}
pub enum RunSet {
    All,
    Specific(Vec<String>),
    Exclude(Vec<String>),
    ExcludeIndexed(Vec<u32>),
    First(u32),
}

fn handle_action(
    runtime: &mut Runtime,
    module_instance: &Option<Rc<ModuleInstance>>,
    action: Action,
    logger: &PrintLogger,
) -> Result<Option<Vec<Value>>> {
    let module_instance = match module_instance {
        Some(mi) => mi,
        None => return err!("action invoked with no module"),
    };
    match action {
        Action::Invoke { id, name, params } => {
            logger.log("SPEC", || {
                format!("INVOKE ACTION {:?} {} {:?}", id, name, params)
            });
            let values: Vec<Value> = params.into_iter().map(|p| p.into()).collect();
            Ok(Some(
                runtime
                    .call(&module_instance, &name, &values)
                    .wrap("calling")?,
            ))
        }
        Action::Get { id, name } => {
            logger.log("SPEC", || format!("GET ACTION {:?} {}", id, name));
            Ok(Some(vec![Value::Num(Num::I32(0))]))
        }
    }
}

pub trait TestCompare<T> {
    fn same_bits(&self, other: &T) -> bool;
}

impl TestCompare<Value> for Value {
    fn same_bits(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Num(Num::F32(a)), Value::Num(Num::F32(b))) => a.to_bits() == b.to_bits(),
            (Value::Num(Num::F64(a)), Value::Num(Num::F64(b))) => a.to_bits() == b.to_bits(),
            _ => self == other,
        }
    }
}

pub fn verify_result(results: Vec<Value>, expects: Vec<ActionResult>) -> Result<()> {
    if results.len() != expects.len() {
        return err!("Expect {} results but got {}", expects.len(), results.len());
    }

    let mut expects = expects;
    for result in results {
        let expect = expects.pop().unwrap();
        match expect {
            ActionResult::NumPat(NumPat::Num(num)) => {
                let expect: Value = num.into();
                if !result.same_bits(&expect) {
                    return err!("Expected {:?}, got {:?}", expect, result);
                }
            }
            ActionResult::NumPat(NumPat::NaNPat(nanpat)) => {
                let resultnum = match result.as_num() {
                    Some(n) => n,
                    _ => return err!("Expected num result, got {:?}", result),
                };
                if !nanpat.accepts(resultnum) {
                    return err!("Expected result type {:?}, got {:?}", nanpat, result);
                }
            }
            ActionResult::Func => {
                if !matches!(result, Value::Ref(Ref::Func(_))) {
                    return err!("Expected Func, got {:?}", result);
                }
            }
            ActionResult::Extern => {
                if !matches!(result, Value::Ref(Ref::Extern(_))) {
                    return err!("Expected Extern, got {:?}", result);
                }
            }
        }
    }
    Ok(())
}

pub struct Failure {
    location: Location,
    testindex: u32,
    err: Error,
}

#[derive(Default)]
pub struct Failures {
    pub failures: Vec<Failure>,
}

impl std::error::Error for Failures {}

impl std::fmt::Debug for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Test {} Line {}", self.testindex, self.location.line)?;
        writeln!(f, "{}\n", self.err)
    }
}

impl std::fmt::Display for Failures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl std::fmt::Debug for Failures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for failure in &self.failures {
            writeln!(f, "{:?}", failure)?;
        }
        writeln!(f, "{} failures", self.failures.len())
    }
}

pub fn run_spec_test(script: SpecTestScript, runset: RunSet) -> Result<()> {
    let mut runtime = Runtime::new();

    let spectest_module = runtime.load(make_spectest_module()).wrap("loading")?;
    runtime.register("spectest", spectest_module);

    let mut module: Option<Rc<ModuleInstance>> = None;

    let mut assert_returns = 0;

    let logger = PrintLogger::default();

    let mut failures: Failures = Failures::default();

    for cmd in script.cmds {
        logger.log("SPEC", || format!("EXECUTE CMD {:?}", cmd));
        match cmd.cmd {
            Cmd::Module(m) => match m {
                Module::Module(m) => {
                    module = Some(runtime.load(m).wrap("loading")?);
                }
                Module::Binary(b) => {
                    let data: Box<[u8]> = b
                        .into_iter()
                        .flat_map(|d| d.into_boxed_bytes().into_vec())
                        .collect();
                    let m = parse(&mut data.as_ref()).wrap("parsing binary")?;
                    module = Some(runtime.load(m).wrap("loading")?);
                }
                Module::Quote(_) => println!("QUOTE MODULE ACTION"),
            },
            Cmd::Register { modname, id } => println!("REGISTER {} {:?}", modname, id),
            Cmd::Action(a) => {
                handle_action(&mut runtime, &module, a, &logger)?;
            }
            Cmd::Assertion(a) => {
                println!("ACTION {:?}", a);
                if let Assertion::Return { action, results } = a {
                    assert_returns += 1;
                    println!("ASSERT RETURN {}", assert_returns);
                    match &runset {
                        RunSet::All => (),
                        RunSet::First(n) => {
                            if assert_returns > *n {
                                return Ok(());
                            }
                        }
                        RunSet::ExcludeIndexed(set) => {
                            if set.iter().any(|i| *i == assert_returns) {
                                continue;
                            }
                        }
                        RunSet::Specific(set) => {
                            if set.iter().find(|i| *i == action.name()).is_none() {
                                continue;
                            }
                        }
                        RunSet::Exclude(set) => {
                            if set.iter().any(|i| *i == action.name()) {
                                continue;
                            }
                        }
                    }
                    let result = handle_action(&mut runtime, &module, action, &logger)?;
                    if let Some(result) = result {
                        match verify_result(result, results) {
                            Ok(_) => (),
                            Err(e) => failures.failures.push(Failure {
                                location: cmd.location,
                                testindex: assert_returns,
                                err: e,
                            }),
                        }
                    }
                }
            }
            Cmd::Meta(m) => println!("META {:?}", m),
        }
    }

    if !failures.failures.is_empty() {
        return Err(failures.wrap("some tests failed"));
    }
    Ok(())
}
