use {
    crate::{
        error::{CmdError, Failure, Result, SpecTestError, TestFailureError},
        format::{Action, ActionResult, Assertion, Cmd, CmdEntry, Module, NumPat, SpecTestScript},
        spectest_module::make_spectest_module,
    },
    std::{collections::HashMap, io::ErrorKind, rc::Rc},
    wrausmt_format::{
        binary::{
            error::{BinaryParseError, BinaryParseErrorKind},
            leb128::LEB128Error,
        },
        loader::{Loader, LoaderError},
        text::{
            parse::error::{ParseError, ParseErrorKind},
            resolve::ResolveError,
            string::WasmString,
        },
    },
    wrausmt_runtime::{
        logger::{Logger, PrintLogger, Tag},
        runtime::{
            error::{RuntimeError, RuntimeErrorKind, TrapKind},
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
                Cmd::Assertion(Assertion::Malformed { failure, .. }) => {
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

trait MalformedMatch {
    fn matches_malformed(&self, err: &CmdError) -> bool;
}

impl MalformedMatch for str {
    fn matches_malformed(&self, err: &CmdError) -> bool {
        let parse_err =
            if let CmdError::LoaderError(LoaderError::ParseError(ParseError { kind, .. })) = err {
                Some(kind)
            } else {
                None
            };
        let bin_parse_err =
            if let CmdError::LoaderError(LoaderError::BinaryParseError(BinaryParseError {
                kind,
                ..
            })) = err
            {
                Some(kind)
            } else {
                None
            };
        match self {
            "alignment" => matches!(parse_err, Some(ParseErrorKind::InvalidAlignment(_))),
            "i32 constant" => matches!(parse_err, Some(ParseErrorKind::ParseIntError(_))),
            "unknown label" => matches!(parse_err, Some(ParseErrorKind::ResolveError(_))),
            "unexpected token" => matches!(
                parse_err,
                // This should really only be unexpected token, but blocks end up parsing
                // out-of-order param/result/type as instructions. One approach to improve this
                // coudl be to define all of the non-instruction keywords as their own tokens.
                Some(ParseErrorKind::UnexpectedToken(_))
                    | Some(ParseErrorKind::UnrecognizedInstruction(_))
            ),
            _ if self.starts_with("unknown operator") => {
                matches!(
                    parse_err,
                    Some(ParseErrorKind::UnexpectedToken(_))
                        | Some(ParseErrorKind::UnrecognizedInstruction(_))
                )
            }
            "integer too large" => matches!(
                bin_parse_err,
                Some(BinaryParseErrorKind::LEB128Error(LEB128Error::Overflow(_)))
            ),
            "integer representation too long" => matches!(
                bin_parse_err,
                Some(BinaryParseErrorKind::LEB128Error(
                    LEB128Error::Unterminated(_)
                )) | Some(BinaryParseErrorKind::InvalidFuncType(_))
            ),
            "magic header not detected" => {
                matches!(bin_parse_err, Some(BinaryParseErrorKind::IncorrectMagic(_)))
            }
            "unknown binary version" => {
                matches!(
                    bin_parse_err,
                    Some(BinaryParseErrorKind::IncorrectVersion(_))
                )
            }
            "unexpected end" => {
                matches!(
                    bin_parse_err,
                    Some(BinaryParseErrorKind::IOError(e)) if e.kind() == ErrorKind::UnexpectedEof
                )
            }
            "inline function type" => matches!(
                parse_err,
                Some(ParseErrorKind::ResolveError(
                    ResolveError::DuplicateTypeIndex(_)
                ))
            ),
            "malformed UTF-8 encoding" => {
                matches!(parse_err, Some(ParseErrorKind::Utf8Error(_)))
                    || matches!(bin_parse_err, Some(BinaryParseErrorKind::Utf8Error(_e)))
            }

            "import after function" => matches!(
                parse_err,
                Some(ParseErrorKind::ResolveError(
                    ResolveError::ImportAfterFunction
                ))
            ),
            "import after global" => matches!(
                parse_err,
                Some(ParseErrorKind::ResolveError(
                    ResolveError::ImportAfterGlobal
                ))
            ),
            "import after table" => matches!(
                parse_err,
                Some(ParseErrorKind::ResolveError(ResolveError::ImportAfterTable))
            ),
            "import after memory" => matches!(
                parse_err,
                Some(ParseErrorKind::ResolveError(
                    ResolveError::ImportAfterMemory
                ))
            ),
            "constant out of range" => matches!(parse_err, Some(ParseErrorKind::InvalidNaN(_))),
            _ => false,
        }
    }
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
                    let resultnum = match result.as_num() {
                        Some(n) => n,
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

    fn verify_trap_result<T>(result: CmdResult<T>, failure: String) -> TestResult<()> {
        match result {
            Err(CmdError::InvocationError(RuntimeError {
                kind: RuntimeErrorKind::Trap(trap_kind),
                ..
            })) if failure.matches_trap(&trap_kind) => Ok(()),
            Err(e) => Err(TestFailureError::TrapMismatch {
                result: Some(Box::new(e)),
                expect: failure,
            }),
            _ => Err(TestFailureError::TrapMismatch {
                result: None,
                expect: failure,
            }),
        }
    }

    fn verify_malformed_result<T>(result: CmdResult<T>, failure: String) -> TestResult<()> {
        match result {
            Err(e) if failure.matches_malformed(&e) => Ok(()),
            Err(e) => Err(TestFailureError::FailureMismatch {
                result: Some(Box::new(e)),
                expect: failure,
            }),
            _ => Err(TestFailureError::FailureMismatch {
                result: None,
                expect: failure,
            }),
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
                        Self::verify_trap_result(result, failure).map_err(|e| e.into())
                    }
                    Assertion::ModuleTrap { module, failure } => {
                        let result = self.handle_module(module);
                        Self::verify_trap_result(result, failure).map_err(|e| e.into())
                    }
                    Assertion::Malformed { module, failure } => {
                        let result = self.handle_module(module);
                        Self::verify_malformed_result(result, failure).map_err(|e| e.into())
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
