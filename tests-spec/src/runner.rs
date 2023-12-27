use {
    super::{
        error::{Failure, Failures, Result, SpecTestError},
        format::{Action, ActionResult, CmdEntry, NumPat, SpecTestScript},
        spectest_module::make_spectest_module,
    },
    crate::format::{Assertion, Cmd, Module},
    format::{loader::Loader, text::string::WasmString},
    std::{collections::HashMap, io::Cursor, rc::Rc},
    wrausmt_runtime::{
        logger::{Logger, PrintLogger, Tag},
        runtime::{
            error::TrapKind,
            instance::ModuleInstance,
            values::{Num, Ref, Value},
            Runtime,
        },
        syntax::{types::RefType, Id},
    },
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
    SpecificIndex(Vec<usize>),
    Exclude(Vec<String>),
    ExcludeIndexed(Vec<usize>),
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
}

#[derive(Debug, Default)]
pub struct SpecTestRunner {
    runtime:       Runtime,
    latest_module: Option<Rc<ModuleInstance>>,
    named_modules: HashMap<Id, Rc<ModuleInstance>>,
    logger:        PrintLogger,
}

trait TrapMatch {
    fn matches_trap(&self, trap: &TrapKind) -> bool;
}

impl TrapMatch for str {
    fn matches_trap(&self, trap: &TrapKind) -> bool {
        match self {
            "invalid conversion to integer" => matches!(trap, TrapKind::InvalidConversionToInteger),
            "out of bounds memory access" => {
                matches!(trap, TrapKind::OutOfBoundsMemoryAccess(..))
            }
            "out of bounds table access" => matches!(trap, TrapKind::OutOfBoundsTableAccess(..)),
            "indirect call type mismatch" => matches!(trap, TrapKind::CallIndirectTypeMismatch),
            "integer divide by zero" => matches!(trap, TrapKind::IntegerDivideByZero),
            "integer overflow" => matches!(trap, TrapKind::IntegerOverflow),
            "unreachable" => matches!(trap, TrapKind::Unreachable),
            "undefined element" => matches!(trap, TrapKind::OutOfBoundsTableAccess(..)),
            "uninitialized element" => matches!(trap, TrapKind::UninitializedElement),
            "uninitialized element 2" => matches!(trap, TrapKind::UninitializedElement),
            _ => false,
        }
    }
}

fn module_data(strings: Vec<WasmString>) -> Box<[u8]> {
    strings
        .into_iter()
        .flat_map(|d| d.into_boxed_bytes().into_vec())
        .collect()
}

fn module_cursor(strings: Vec<WasmString>) -> Cursor<Box<[u8]>> {
    Cursor::new(module_data(strings))
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

    fn module_for_action(&self, modname: &Option<Id>) -> Result<Rc<ModuleInstance>> {
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

    fn handle_module(&mut self, m: Module) -> Result<(Option<Id>, Rc<ModuleInstance>)> {
        match m {
            Module::Module(m) => Ok((m.id.clone(), self.runtime.load(m)?)),
            Module::Binary(n, b) => {
                let mut data = module_cursor(b);
                Ok((n, self.runtime.load_wasm_data(&mut data)?))
            }
            Module::Quote(n, b) => {
                let mut data = module_cursor(b);
                Ok((n, self.runtime.load_wast_data(&mut data)?))
            }
        }
    }

    fn verify_trap_result<T>(result: Result<T>, failure: String) -> Result<()> {
        match result {
            Err(e) => {
                let trap_error = e.as_trap_error();
                match trap_error {
                    Some(tk) if failure.matches_trap(tk) => Ok(()),
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
                    Err(_) => return Err(SpecTestError::RegisterMissingModule(modname)),
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
                        Self::verify_result(result, results)
                    }
                    Assertion::ActionTrap { action, failure } => {
                        if !runset.should_run_name(action.name()) {
                            return Ok(());
                        }
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
                        // let _ = self.handle_module(module);
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
                        // let _ = self.handle_module(module);
                        // TODO verify result
                        Ok(())
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
            .map_err(|e| e.into_failure(cmd_entry.location, test_index))
            .err()
    }

    pub fn run_spec_test(mut self, script: SpecTestScript, runset: RunSet) -> Result<()> {
        let failures: Failures = Failures {
            failures: script
                .cmds
                .into_iter()
                .enumerate()
                .filter(|(idx, _)| runset.should_run_index(idx))
                .filter_map(|(idx, cmd_entry)| self.log_and_run_command(idx, cmd_entry, &runset))
                .collect(),
        };

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