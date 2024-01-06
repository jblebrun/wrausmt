use {
    crate::{
        error::{CmdError, Failure, Result, SpecTestError, TestFailureError},
        error_mappings::verify_failure,
        format::{Action, ActionResult, Assertion, Cmd, CmdEntry, Module, NumPat, SpecTestScript},
        spectest_module::make_spectest_module,
    },
    std::{
        collections::HashMap,
        panic::{catch_unwind, PanicInfo},
        rc::Rc,
    },
    wrausmt_format::{loader::Loader, text::string::WasmString},
    wrausmt_runtime::{
        logger::{Logger, PrintLogger, Tag},
        runtime::{
            instance::ModuleInstance,
            values::{Num, Ref, Value},
            Runtime,
        },
        syntax::{types::RefType, Id},
    },
};

pub type CmdResult<T> = std::result::Result<T, CmdError>;
pub type TestResult<T> = std::result::Result<T, TestFailureError>;

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

static GLOBAL_FAILURES_TO_IGNORE: &[&str] = &[
    "alignment must not be larger than natural",
    "constant expression required",
    "duplicate export name",
    "global is immutable",
    "invalid result arity",
    "memory size must be at most 65536 pages (4GiB)",
    "multiple memories",
    "size minimum must not be greater than maximum",
    "start function",
    "type mismatch",
    "undeclared function reference",
    "unknown data segment",
    "unknown data segment 1",
    "unknown elem segment 0",
    "unknown elem segment 4",
    "unknown function",
    "unknown function 7",
    "unknown global",
    "unknown global 0",
    "unknown global 1",
    "unknown label",
    "unknown local",
    "unknown memory",
    "unknown memory 0",
    "unknown memory 1",
    "unknown table",
    "unknown table 0",
    "unknown type",
];

pub enum RunSet {
    All,
    Specific(Vec<String>),
    SpecificIndex(Vec<usize>),
    Exclude(Vec<String>),
    ExcludeIndexed(Vec<usize>),
    ExcludeFailure(Vec<String>),
    First(usize),
}

impl RunSet {
    fn should_run_name(&self, name: &str) -> bool {
        match self {
            RunSet::Specific(set) => set.iter().any(|i| *i == name),
            RunSet::Exclude(set) => !set.iter().any(|i| *i == name),
            _ => true,
        }
    }

    fn should_run_index(&self, index: &usize) -> bool {
        match self {
            RunSet::First(n) => index <= n,
            RunSet::SpecificIndex(set) => set.iter().any(|i| i == index),
            RunSet::ExcludeIndexed(set) => !set.iter().any(|i| i == index),
            _ => true,
        }
    }

    fn should_run_cmd(&self, cmd: &Cmd) -> bool {
        match self {
            RunSet::ExcludeFailure(fs) => match cmd {
                Cmd::Assertion(Assertion::Invalid { failure, .. })
                | Cmd::Assertion(Assertion::Malformed { failure, .. }) => {
                    !fs.iter().any(|f| failure.starts_with(f))
                }
                _ => true,
            },
            _ => true,
        }
    }
}

#[derive(Debug, Default)]
pub struct SpecTestRunner {
    runtime:       Runtime,
    latest_module: Option<Rc<ModuleInstance>>,
    named_modules: HashMap<Id, Rc<ModuleInstance>>,
    logger:        PrintLogger,
}

fn module_data(strings: Vec<WasmString>) -> Box<[u8]> {
    strings
        .into_iter()
        .flat_map::<Vec<u8>, _>(|d| d.into())
        .collect()
}

impl SpecTestRunner {
    pub fn new() -> Self {
        let mut runtime = Runtime::new();
        let spectest_module = runtime.load(make_spectest_module().unwrap()).unwrap();
        runtime.register("spectest", spectest_module);
        SpecTestRunner {
            runtime,
            ..Self::default()
        }
    }

    fn module_for_action(&self, modname: &Option<Id>) -> CmdResult<Rc<ModuleInstance>> {
        match modname {
            Some(name) => self.named_modules.get(name).cloned(),
            None => self.latest_module.clone(),
        }
        .ok_or_else(|| CmdError::NoModule(modname.clone()))
    }

    fn handle_action(&mut self, action: Action) -> CmdResult<Vec<Value>> {
        match action {
            Action::Invoke {
                modname,
                name,
                params,
            } => {
                let module_instance = self.module_for_action(&modname)?;
                self.logger.log(Tag::Spec, || {
                    format!("INVOKE ACTION {:?} {} {:?}", modname, name, params)
                });
                let values: Vec<Value> = params.into_iter().map(|p| p.into()).collect();
                Ok(self.runtime.call(&module_instance, &name, &values)?)
            }
            Action::Get { modname, name } => {
                let module_instance = self.module_for_action(&modname)?;
                self.logger
                    .log(Tag::Spec, || format!("GET ACTION {:?} {}", modname, name));
                Ok(vec![self.runtime.get_global(&module_instance, &name)?])
            }
        }
    }

    pub fn verify_result(results: Vec<Value>, expects: Vec<ActionResult>) -> TestResult<()> {
        if results.len() != expects.len() {
            return Err(TestFailureError::ResultLengthMismatch { results, expects });
        }

        let mut expects = expects;
        for result in results {
            let expect = expects.pop().unwrap();
            match expect {
                ActionResult::NumPat(NumPat::Num(num)) => {
                    let expectnum: Value = num.into();
                    if !result.same_bits(&expectnum) {
                        return Err(TestFailureError::ResultMismatch { result, expect });
                    }
                }
                ActionResult::NumPat(NumPat::NaNPat(nanpat)) => {
                    let resultnum = match result {
                        Value::Num(n) => n,
                        _ => return Err(TestFailureError::ResultMismatch { result, expect }),
                    };
                    if !nanpat.accepts(resultnum) {
                        return Err(TestFailureError::ResultMismatch { result, expect });
                    }
                }
                ActionResult::Func => {
                    if !matches!(
                        result,
                        Value::Ref(Ref::Null(RefType::Func)) | Value::Ref(Ref::Func(_))
                    ) {
                        return Err(TestFailureError::ResultMismatch { result, expect });
                    }
                }
                ActionResult::Extern => {
                    if !matches!(
                        result,
                        Value::Ref(Ref::Null(RefType::Extern)) | Value::Ref(Ref::Extern(_))
                    ) {
                        return Err(TestFailureError::ResultMismatch { result, expect });
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_module(&mut self, m: Module) -> CmdResult<(Option<Id>, Rc<ModuleInstance>)> {
        match m {
            Module::Module(m) => Ok((m.id.clone(), self.runtime.load(m)?)),
            Module::Binary(n, b) => {
                let data = module_data(b);
                Ok((n, self.runtime.load_wasm_data(&mut data.as_ref())?))
            }
            Module::Quote(n, b) => {
                let data = module_data(b);
                Ok((n, self.runtime.load_wast_data(&mut data.as_ref())?))
            }
        }
    }

    fn run_cmd_entry(&mut self, cmd: Cmd, runset: &RunSet) -> CmdResult<()> {
        self.logger
            .log(Tag::Spec, || format!("EXECUTE CMD {:?}", cmd));
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
                self.logger.log(Tag::SpecModule, || {
                    format!("REGISTER {} {:?}", modname, module)
                });
                match module {
                    Ok(module) => self.runtime.register(modname, module.clone()),
                    Err(_) => return Err(CmdError::RegisterMissingModule(modname)),
                }
                Ok(())
            }
            Cmd::Action(a) => {
                self.handle_action(a)?;
                Ok(())
            }
            Cmd::Assertion(a) => {
                self.logger.log(Tag::Spec, || format!("ACTION {:?}", a));
                match a {
                    Assertion::Return { action, results } => {
                        if !runset.should_run_name(action.name()) {
                            return Ok(());
                        }
                        let result = self.handle_action(action)?;
                        Self::verify_result(result, results).map_err(|e| e.into())
                    }
                    Assertion::ActionTrap { action, failure } => {
                        if !runset.should_run_name(action.name()) {
                            return Ok(());
                        }
                        let result = self.handle_action(action);
                        verify_failure(result, &failure).map_err(|e| e.into())
                    }
                    Assertion::ModuleTrap { module, failure } => {
                        let result = self.handle_module(module);
                        verify_failure(result, &failure).map_err(|e| e.into())
                    }
                    Assertion::Malformed { module, failure } => {
                        let result = self.handle_module(module);
                        verify_failure(result, &failure).map_err(|e| e.into())
                    }
                    Assertion::Exhaustion { action, failure } => {
                        let result = self.handle_action(action);
                        verify_failure(result, &failure).map_err(|e| e.into())
                    }
                    Assertion::Unlinkable { module, failure } => {
                        let result = self.handle_module(module);
                        verify_failure(result, &failure).map_err(|e| e.into())
                    }
                    Assertion::Invalid { module, failure } => {
                        if GLOBAL_FAILURES_TO_IGNORE.contains(&failure.as_str()) {
                            return Ok(());
                        }
                        let result = unsafe {
                            let pself = self as *mut Self;
                            catch_unwind(|| (*pself).handle_module(module))
                        };
                        match result {
                            Ok(result) => verify_failure(result, &failure).map_err(|e| e.into()),
                            Err(p) => {
                                if let Some(pi) = p.downcast_ref::<PanicInfo>() {
                                    Err(TestFailureError::Panic(pi.to_string()).into())
                                } else if let Some(m) = p.downcast_ref::<&'static str>() {
                                    Err(TestFailureError::Panic(m.to_string()).into())
                                } else if let Some(m) = p.downcast_ref::<String>() {
                                    Err(TestFailureError::Panic(m.to_string()).into())
                                } else {
                                    Err(TestFailureError::Panic("dunno".to_string()).into())
                                }
                            }
                        }
                    }
                }
            }
            Cmd::Meta(m) => {
                self.logger.log(Tag::Spec, || format!("META{:?}", m));
                Ok(())
            }
        }
    }

    pub fn log_and_run_command(
        &mut self,
        test_index: usize,
        cmd_entry: CmdEntry,
        runset: &RunSet,
    ) -> Option<Failure> {
        self.logger.log(Tag::Spec, || {
            format!(
                "*****BEGIN TEST #{}****** ({:?})",
                test_index, cmd_entry.location
            )
        });
        let now = std::time::Instant::now();
        let result = self.run_cmd_entry(cmd_entry.cmd, runset);
        self.logger.log(Tag::Spec, || {
            format!(
                "*****END TEST #{}****** ({}ms)\n",
                test_index,
                now.elapsed().as_secs_f32() * 1000.0
            )
        });
        result
            .map_err(|e| Failure {
                location: cmd_entry.location,
                test_index,
                err: e,
            })
            .err()
    }

    pub fn run_spec_test(mut self, script: SpecTestScript, runset: RunSet) -> Result<()> {
        let failures: Vec<Failure> = script
            .cmds
            .into_iter()
            .enumerate()
            .filter(|(idx, cmd_entry)| {
                runset.should_run_index(idx) && runset.should_run_cmd(&cmd_entry.cmd)
            })
            .filter_map(|(idx, cmd_entry)| self.log_and_run_command(idx, cmd_entry, &runset))
            .collect();

        if !failures.is_empty() {
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
