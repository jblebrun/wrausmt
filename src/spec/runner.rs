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
        error::TrapKind,
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

impl From<&str> for TrapKind {
    fn from(msg: &str) -> Self {
        match msg {
            "invalid conversion to integer" => TrapKind::InvalidConversionToInteger,
            "out of bounds memory access" => TrapKind::OutOfBoundsMemoryAccess,
            "out of bounds table access" => TrapKind::OutOfBoundsTableAccess,
            "indirect call type mismatch" => TrapKind::CallIndirectTypeMismatch,
            "integer divide by zero" => TrapKind::IntegerDivideByZero,
            "integer overflow" => TrapKind::IntegerOverflow,
            "unreachable" => TrapKind::Unreachable,
            "undefined element" => TrapKind::OutOfBoundsTableAccess,
            "uninitialized element" => TrapKind::UninitializedElement,
            "uninitialized element 2" => TrapKind::UninitializedElement,
            _ => panic!("don't know how to convert {}", msg),
        }
    }
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

    fn handle_module(&mut self, m: Module) -> Result<(Option<String>, Rc<ModuleInstance>)> {
        match m {
            Module::Module(m) => Ok((m.id.clone(), self.runtime.load(m)?)),
            Module::Binary(n, b) => {
                let data: Box<[u8]> = b
                    .into_iter()
                    .flat_map(|d| d.into_boxed_bytes().into_vec())
                    .collect();
                Ok((n, self.runtime.load_wasm_data(&data)?))
            }
            Module::Quote(_, _) => {
                // TODO parse quote modules
                Err(SpecTestError::UnImplemented)
            }
        }
    }

    fn verify_trap_result<T>(result: Result<T>, failure: String) -> Result<()> {
        let expected_trap: TrapKind = failure.as_str().into();
        println!("EXPECT: {:?}", expected_trap);
        match result {
            Err(e) => {
                let trap_error = e.as_trap_error();
                match trap_error {
                    Some(tk) if tk == &expected_trap => Ok(()),
                    _ => Err(SpecTestError::TrapMismatch {
                        result: Some(Box::new(e)),
                        expect: failure,
                    }),
                }
            }
            _ => Err(SpecTestError::TrapMismatch {
                result: None,
                expect: failure,
            }),
        }
    }

    fn run_cmd_entry(&mut self, cmd: Cmd, runset: &RunSet) -> Result<()> {
        self.logger.log("SPEC", || format!("EXECUTE CMD {:?}", cmd));
        match cmd {
            Cmd::Module(m) => {
                let (name, modinst) = self.handle_module(m)?;
                if let Some(name) = name {
                    self.named_modules.insert(name, modinst.clone());
                }
                self.latest_module = Some(modinst);
                Ok(())
            }
            Cmd::Register { modname, id } => {
                let module = self.module_for_action(&id);
                println!("REGISTER {} {:?}", modname, module);
                match module {
                    Ok(module) => self.runtime.register(modname, module.clone()),
                    Err(_) => return Err(SpecTestError::RegisterMissingModule(modname)),
                }
                Ok(())
            }
            Cmd::Action(a) => {
                self.handle_action(a)?;
                Ok(())
            }
            Cmd::Assertion(a) => {
                println!("ACTION {:?}", a);
                match a {
                    Assertion::Return { action, results } => {
                        self.assert_returns += 1;
                        if !runset.should_run(action.name(), self.assert_returns) {
                            return Ok(());
                        }
                        let result = self.handle_action(action)?;
                        Self::verify_result(result, results)
                    }
                    Assertion::ActionTrap { action, failure } => {
                        let result = self.handle_action(action);
                        Self::verify_trap_result(result, failure)
                    }
                    Assertion::ModuleTrap { module, failure } => {
                        let result = self.handle_module(module);
                        Self::verify_trap_result(result, failure)
                    }
                    Assertion::Malformed {
                        module: _,
                        failure: _,
                    } => {
                        //let _ = self.handle_module(module);
                        // TODO verify result
                        Ok(())
                    }
                    Assertion::Exhaustion { action, failure: _ } => {
                        let _ = self.handle_action(action);
                        // TODO verify result
                        Ok(())
                    }
                    Assertion::Unlinkable { module, failure: _ } => {
                        let _ = self.handle_module(module);
                        // TODO verify result
                        Ok(())
                    }
                    Assertion::Invalid {
                        module: _,
                        failure: _,
                    } => {
                        //let _ = self.handle_module(module);
                        // TODO verify result
                        Ok(())
                    }
                }
            }
            Cmd::Meta(m) => {
                println!("META {:?}", m);
                Ok(())
            }
        }
    }

    pub fn run_spec_test(mut self, script: SpecTestScript, runset: RunSet) -> Result<()> {
        let mut failures: Failures = Failures::default();
        for cmd_entry in script.cmds {
            match self.run_cmd_entry(cmd_entry.cmd, &runset) {
                Ok(()) => {}
                Err(e) => failures
                    .failures
                    .push(e.into_failure(cmd_entry.location, self.assert_returns)),
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
