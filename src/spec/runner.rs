use std::{collections::HashMap, rc::Rc};

use super::{
    error::{Failures, Result, SpecTestError},
    format::{Action, ActionResult, NumPat, SpecTestScript},
    spectest_module::make_spectest_module,
};
use crate::{
    loader::Loader,
    logger::{Logger, PrintLogger},
    runtime::{
        instance::ModuleInstance,
        values::{Num, Ref, Value},
        Runtime,
    },
    spec::format::{Assertion, Cmd, Module},
    types::RefType,
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
    SpecificIndex(Vec<u32>),
    Exclude(Vec<String>),
    ExcludeIndexed(Vec<u32>),
    First(u32),
}

impl RunSet {
    fn should_run(&self, name: &str, index: u32) -> bool {
        match self {
            RunSet::All => true,
            RunSet::First(n) => index <= *n,
            RunSet::SpecificIndex(set) => set.iter().any(|i| *i == index),
            RunSet::ExcludeIndexed(set) => !set.iter().any(|i| *i == index),
            RunSet::Specific(set) => set.iter().any(|i| *i == name),
            RunSet::Exclude(set) => !set.iter().any(|i| *i == name),
        }
    }
}

#[derive(Debug, Default)]
pub struct SpecTestRunner {
    runtime: Runtime,
    latest_module: Option<Rc<ModuleInstance>>,
    named_modules: HashMap<String, Rc<ModuleInstance>>,
    assert_returns: u32,
    logger: PrintLogger,
}

impl SpecTestRunner {
    pub fn new() -> Self {
        let mut runtime = Runtime::new();
        let spectest_module = runtime.load(make_spectest_module()).unwrap();
        runtime.register("spectest", spectest_module);
        SpecTestRunner {
            runtime,
            ..Self::default()
        }
    }

    fn module_for_action(&self, modname: &Option<String>) -> Result<Rc<ModuleInstance>> {
        match modname {
            Some(name) => self.named_modules.get(name).cloned(),
            None => self.latest_module.clone(),
        }
        .ok_or_else(|| SpecTestError::NoModule(modname.clone()))
    }

    fn handle_action(&mut self, action: Action) -> Result<Vec<Value>> {
        match action {
            Action::Invoke {
                modname,
                name,
                params,
            } => {
                let module_instance = self.module_for_action(&modname)?;
                self.logger.log("SPEC", || {
                    format!("INVOKE ACTION {:?} {} {:?}", modname, name, params)
                });
                let values: Vec<Value> = params.into_iter().map(|p| p.into()).collect();
                Ok(self.runtime.call(&module_instance, &name, &values)?)
            }
            Action::Get { modname, name } => {
                let module_instance = self.module_for_action(&modname)?;
                self.logger
                    .log("SPEC", || format!("GET ACTION {:?} {}", modname, name));
                Ok(vec![self.runtime.get_global(&module_instance, &name)?])
            }
        }
    }

    pub fn verify_result(results: Vec<Value>, expects: Vec<ActionResult>) -> Result<()> {
        if results.len() != expects.len() {
            return Err(SpecTestError::ResultLengthMismatch { results, expects });
        }

        let mut expects = expects;
        for result in results {
            let expect = expects.pop().unwrap();
            match expect {
                ActionResult::NumPat(NumPat::Num(num)) => {
                    let expectnum: Value = num.into();
                    if !result.same_bits(&expectnum) {
                        return Err(SpecTestError::ResultMismatch { result, expect });
                    }
                }
                ActionResult::NumPat(NumPat::NaNPat(nanpat)) => {
                    let resultnum = match result.as_num() {
                        Some(n) => n,
                        _ => return Err(SpecTestError::ResultMismatch { result, expect }),
                    };
                    if !nanpat.accepts(resultnum) {
                        return Err(SpecTestError::ResultMismatch { result, expect });
                    }
                }
                ActionResult::Func => {
                    if !matches!(
                        result,
                        Value::Ref(Ref::Null(RefType::Func)) | Value::Ref(Ref::Func(_))
                    ) {
                        return Err(SpecTestError::ResultMismatch { result, expect });
                    }
                }
                ActionResult::Extern => {
                    if !matches!(
                        result,
                        Value::Ref(Ref::Null(RefType::Extern)) | Value::Ref(Ref::Extern(_))
                    ) {
                        return Err(SpecTestError::ResultMismatch { result, expect });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn run_spec_test(mut self, script: SpecTestScript, runset: RunSet) -> Result<()> {
        let mut failures: Failures = Failures::default();
        for cmd in script.cmds {
            self.logger.log("SPEC", || format!("EXECUTE CMD {:?}", cmd));
            match cmd.cmd {
                Cmd::Module(m) => {
                    let (name, modinst) = match m {
                        Module::Module(m) => (m.id.clone(), self.runtime.load(m)?),
                        Module::Binary(n, b) => {
                            let data: Box<[u8]> = b
                                .into_iter()
                                .flat_map(|d| d.into_boxed_bytes().into_vec())
                                .collect();
                            (n, self.runtime.load_wasm_data(&data)?)
                        }
                        Module::Quote(_, _) => {
                            panic!("no quote module support yet")
                        }
                    };
                    if let Some(name) = name {
                        self.named_modules.insert(name, modinst.clone());
                    }
                    self.latest_module = Some(modinst);
                }
                Cmd::Register { modname, id } => {
                    let module = self.module_for_action(&id);
                    println!("REGISTER {} {:?}", modname, module);
                    match module {
                        Ok(module) => self.runtime.register(modname, module.clone()),
                        Err(_) => return Err(SpecTestError::RegisterMissingModule(modname)),
                    }
                }
                Cmd::Action(a) => {
                    self.handle_action(a)?;
                }
                Cmd::Assertion(a) => {
                    println!("ACTION {:?}", a);
                    if let Assertion::Return { action, results } = a {
                        self.assert_returns += 1;
                        if !runset.should_run(action.name(), self.assert_returns) {
                            return Ok(());
                        }
                        println!("ASSERT RETURN {}", self.assert_returns);
                        let result = self.handle_action(action)?;
                        match Self::verify_result(result, results) {
                            Ok(()) => (),
                            Err(e) => failures
                                .failures
                                .push(e.into_failure(cmd.location, self.assert_returns)),
                        }
                    }
                }
                Cmd::Meta(m) => println!("META {:?}", m),
            }
        }

        if !failures.failures.is_empty() {
            return Err(SpecTestError::Failures(failures));
        }
        Ok(())
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
