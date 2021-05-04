use crate::format::text::parse::error::{ParseError, Result};
use crate::format::text::token::Token;
use crate::syntax as modulesyntax;
use crate::{
    format::text::parse::Parser,
    runtime::values::{Num, Ref, Value},
    syntax::Resolved,
    types::{NumType, RefType},
};
use std::io::Read;

impl<R: Read> Parser<R> {
    pub fn parse_spec_test(&mut self) -> Result<SpecTestScript> {
        match self.zero_or_more(Self::try_cmd) {
            Ok(cmds) => {
                if self.current.token != Token::Eof {
                    return Err(self.with_context(ParseError::unexpected("cmd")));
                }
                Ok(SpecTestScript { cmds })
            }
            Err(e) => Err(self.with_context(e)),
        }
    }

    fn try_cmd(&mut self) -> Result<Option<Cmd>> {
        self.first_of(&[
            Self::try_module_cmd,
            Self::try_register_cmd,
            Self::try_action_cmd,
            Self::try_assertion_cmd,
            Self::try_meta_cmd,
        ])
    }

    fn try_module_cmd(&mut self) -> Result<Option<Cmd>> {
        if let Some(module) = self.try_spec_module()? {
            return Ok(Some(Cmd::Module(module)));
        }

        Ok(None)
    }

    fn try_spec_module(&mut self) -> Result<Option<Module>> {
        if !self.try_expr_start("module")? {
            return Ok(None);
        }

        let kw = self.try_keyword()?;

        match kw.as_deref() {
            Some("binary") => {
                let strings = self.zero_or_more(Self::try_string)?;
                self.expect_close()?;
                Ok(Some(Module::Binary(strings)))
            }
            Some("quote") => {
                let strings = self.zero_or_more(Self::try_string)?;
                self.expect_close()?;
                Ok(Some(Module::Quote(strings)))
            }
            _ => {
                if let Some(module) = self.try_module_rest()? {
                    Ok(Some(Module::Module(module)))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn try_register_cmd(&mut self) -> Result<Option<Cmd>> {
        if !self.try_expr_start("register")? {
            return Ok(None);
        }

        Ok(None)
    }

    fn try_action_cmd(&mut self) -> Result<Option<Cmd>> {
        self.try_action().map(|a| a.map(Cmd::Action))
    }

    fn try_action(&mut self) -> Result<Option<Action>> {
        self.first_of(&[Self::try_invoke_action, Self::try_get_action])
    }

    fn expect_action(&mut self) -> Result<Action> {
        self.try_action()?
            .ok_or_else(|| ParseError::unexpected("spec test action"))
    }

    fn try_invoke_action(&mut self) -> Result<Option<Action>> {
        if !self.try_expr_start("invoke")? {
            return Ok(None);
        }

        let id = self.try_id()?;

        let name = self.expect_string()?;

        let params = self.zero_or_more(Self::try_const)?;

        self.expect_close()?;

        Ok(Some(Action::Invoke { id, name, params }))
    }

    fn try_get_action(&mut self) -> Result<Option<Action>> {
        if !self.try_expr_start("get")? {
            return Ok(None);
        }
        Ok(None)
    }

    fn try_assertion_cmd(&mut self) -> Result<Option<Cmd>> {
        self.first_of(&[
            Self::try_assert_return,
            Self::try_assert_exhaustion,
            Self::try_assert_trap,
            Self::try_assert_invalid,
            Self::try_assert_malformed,
            Self::try_assert_unlinkable,
        ])
        .map(|a| a.map(Cmd::Assertion))
    }

    fn try_meta_cmd(&mut self) -> Result<Option<Cmd>> {
        if self.try_expr_start("script")? {
            let id = self.try_id()?;
            let script = self.expect_string()?;
            self.expect_close()?;
            return Ok(Some(Cmd::Meta(Meta::Script { id, script })));
        }
        if self.try_expr_start("input")? {
            let id = self.try_id()?;
            let file = self.expect_string()?;
            self.expect_close()?;
            return Ok(Some(Cmd::Meta(Meta::Input { id, file })));
        }
        if self.try_expr_start("output")? {
            let id = self.try_id()?;
            let file = self.expect_string()?;
            self.expect_close()?;
            return Ok(Some(Cmd::Meta(Meta::Output { id, file })));
        }

        Ok(None)
    }

    fn try_assert_return(&mut self) -> Result<Option<Assertion>> {
        if !self.try_expr_start("assert_return")? {
            return Ok(None);
        }

        let action = self.expect_action()?;

        let results = self.zero_or_more(Self::try_result)?;

        self.expect_close()?;

        Ok(Some(Assertion::Return { action, results }))
    }

    fn try_assert_exhaustion(&mut self) -> Result<Option<Assertion>> {
        if !self.try_expr_start("assert_exhaustion")? {
            return Ok(None);
        }

        let action = self.expect_action()?;
        let failure = self.expect_string()?;
        self.expect_close()?;

        Ok(Some(Assertion::Exhaustion { action, failure }))
    }

    fn try_assert_trap(&mut self) -> Result<Option<Assertion>> {
        if !self.try_expr_start("assert_trap")? {
            return Ok(None);
        }

        if let Some(action) = self.try_action()? {
            let failure = self.expect_string()?;
            self.expect_close()?;

            return Ok(Some(Assertion::ActionTrap { action, failure }));
        }

        if let Some(module) = self.try_spec_module()? {
            let failure = self.expect_string()?;
            self.expect_close()?;

            return Ok(Some(Assertion::ModuleTrap { module, failure }));
        }

        Ok(None)
    }

    fn try_assert_invalid(&mut self) -> Result<Option<Assertion>> {
        if !self.try_expr_start("assert_invalid")? {
            return Ok(None);
        }

        let module = self
            .try_spec_module()?
            .ok_or_else(|| ParseError::unexpected("spec module"))?;

        let failure = self.expect_string()?;

        self.expect_close()?;

        Ok(Some(Assertion::Invalid { module, failure }))
    }

    fn try_assert_malformed(&mut self) -> Result<Option<Assertion>> {
        if !self.try_expr_start("assert_malformed")? {
            return Ok(None);
        }

        let module = self
            .try_spec_module()?
            .ok_or_else(|| ParseError::unexpected("spec module"))?;

        let failure = self.expect_string()?;

        self.expect_close()?;

        Ok(Some(Assertion::Malformed { module, failure }))
    }

    fn try_assert_unlinkable(&mut self) -> Result<Option<Assertion>> {
        if !self.try_expr_start("assert_invalid")? {
            return Ok(None);
        }

        let module = self
            .try_spec_module()?
            .ok_or_else(|| ParseError::unexpected("spec module"))?;

        let failure = self.expect_string()?;

        self.expect_close()?;

        Ok(Some(Assertion::Unlinkable { module, failure }))
    }

    fn try_result(&mut self) -> Result<Option<ActionResult>> {
        if self.try_expr_start("ref.func")? {
            self.expect_close()?;
            return Ok(Some(ActionResult::Func));
        }

        if self.try_expr_start("ref.extern")? {
            self.expect_close()?;
            return Ok(Some(ActionResult::Extern));
        }

        let num = self.try_num_pat()?;

        match num {
            Some(pat) => Ok(Some(ActionResult::Num(pat))),
            _ => Ok(None),
        }
    }

    fn try_const(&mut self) -> Result<Option<Const>> {
        if self.try_expr_start("ref.null")? {
            let reftype = self.expect_reftype()?;
            self.expect_close()?;
            return Ok(Some(Const::RefNull(reftype)));
        }

        if self.try_expr_start("ref.host")? {
            let val = self.expect_integer()?;
            self.expect_close()?;
            return Ok(Some(Const::RefHost(val)));
        }

        let num = self.try_num_pat()?;

        match num {
            Some(pat) => Ok(Some(Const::Num(pat))),
            _ => Ok(None),
        }
    }

    fn try_num_pat(&mut self) -> Result<Option<NumPat>> {
        let kw = self.peek_next_keyword()?;
        match kw {
            Some(kw) if kw == "nan:canonical" => Ok(Some(NumPat::CanonicalNaN)),
            Some(kw) if kw == "nan:arithmetic" => Ok(Some(NumPat::ArithmeticNaN)),
            Some(kw) if kw.ends_with(".const") => {
                let numtype = kw.split('.').next();
                let nt = match numtype {
                    Some("i32") => Some(NumType::I32),
                    Some("i64") => Some(NumType::I64),
                    Some("f32") => Some(NumType::F32),
                    Some("f64") => Some(NumType::F64),
                    _ => None,
                };
                if let Some(nt) = nt {
                    self.advance()?;
                    self.advance()?;
                    let val = self.expect_number()?;
                    self.expect_close()?;
                    Ok(Some(NumPat::Num(nt, val)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}

/// script: <cmd>*
#[derive(Debug, Default)]
pub struct SpecTestScript {
    pub cmds: Vec<Cmd>,
}

/// cmd:
///   <module>                                   ;; define, validate, and initialize module
///   ( register <string> <name>? )              ;; register module for imports
///   <action>                                   ;; perform action and print results
///   <assertion>                                ;; assert result of an action
///   <meta>
#[derive(Debug)]
pub enum Cmd {
    Module(Module),
    Register { string: String, name: String },
    Action(Action),
    Assertion(Assertion),
    Meta(Meta),
}

/// module:
///   ...
///   ( module <name>? binary <string>* )        ;; module in binary format (may be malformed)
///   ( module <name>? quote <string>* )         ;; module quoted in text (may be malformed)
#[derive(Debug)]
pub enum Module {
    Module(modulesyntax::Module<Resolved>),
    Binary(Vec<String>),
    Quote(Vec<String>),
}

/// action:
///   ( invoke <name>? <string> <const>* )       ;; invoke function export
///   ( get <name>? <string> )                   ;; get global export
#[derive(Debug)]
pub enum Action {
    Invoke {
        id: Option<String>,
        name: String,
        params: Vec<Const>,
    },
    Get {
        id: Option<String>,
        name: String,
    },
}

/// const:
///   ( <num_type>.const <num> )                 ;; number value
///   ( ref.null <ref_kind> )                    ;; null reference
///   ( ref.host <nat> )                         ;; host reference
#[derive(Debug)]
pub enum Const {
    Num(NumPat),
    RefNull(RefType),
    RefHost(u64),
}

impl From<Const> for Value {
    fn from(c: Const) -> Value {
        match c {
            Const::Num(p) => p.into(),
            Const::RefHost(h) => Value::Ref(Ref::Extern(h)),
            Const::RefNull(t) => match t {
                RefType::Func => Value::Ref(Ref::Func(0)),
                RefType::Extern => Value::Ref(Ref::Extern(0)),
            },
        }
    }
}

impl From<NumPat> for Value {
    fn from(np: NumPat) -> Value {
        match np {
            NumPat::Num(t, v) => match t {
                NumType::I32 => Value::Num(Num::I32(v as u32)),
                NumType::I64 => Value::Num(Num::I64(v)),
                NumType::F32 => Value::Num(Num::F32(v as f32)),
                NumType::F64 => Value::Num(Num::F64(v as f64)),
            },
            NumPat::ArithmeticNaN => panic!("not yet"),
            NumPat::CanonicalNaN => panic!("not yet"),
        }
    }
}

/// assertion:
///   ( assert_return <action> <result>* )       ;; assert action has expected results
///   ( assert_trap <action> <failure> )         ;; assert action traps with given failure string
///   ( assert_exhaustion <action> <failure> )   ;; assert action exhausts system resources
///   ( assert_malformed <module> <failure> )    ;; assert module cannot be decoded with given failure string
///   ( assert_invalid <module> <failure> )      ;; assert module is invalid with given failure string
///   ( assert_unlinkable <module> <failure> )   ;; assert module fails to link
///   ( assert_trap <module> <failure> )         ;; assert module traps on instantiation
#[derive(Debug)]
pub enum Assertion {
    Return {
        action: Action,
        results: Vec<ActionResult>,
    },
    ActionTrap {
        action: Action,
        failure: String,
    },
    Exhaustion {
        action: Action,
        failure: String,
    },
    Malformed {
        module: Module,
        failure: String,
    },
    Invalid {
        module: Module,
        failure: String,
    },
    Unlinkable {
        module: Module,
        failure: String,
    },
    ModuleTrap {
        module: Module,
        failure: String,
    },
}

/// result:
///   ( <num_type>.const <num_pat> )
///   ( ref.extern )
///   ( ref.func )
#[derive(Debug)]
pub enum ActionResult {
    Num(NumPat),
    Extern,
    Func,
}

/// num_pat:
///   <value>                                    ;; literal result
///   nan:canonical                              ;; NaN in canonical form
///   nan:arithmetic                             ;; NaN with 1 in MSB of payload
#[derive(Debug)]
pub enum NumPat {
    Num(NumType, u64),
    CanonicalNaN,
    ArithmeticNaN,
}

/// meta:
///  ( script <name>? <script> )                ;; name a subscript
///  ( input <name>? <string> )                 ;; read script or module from file
///  ( output <name>? <string>? )               ;; output module to stout or file
#[derive(Debug)]
pub enum Meta {
    Script { id: Option<String>, script: String },
    Input { id: Option<String>, file: String },
    Output { id: Option<String>, file: String },
}
