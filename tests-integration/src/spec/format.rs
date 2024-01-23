use {
    std::io::Read,
    wrausmt_format::text::{
        parse::{error::Result, pctx, Parser},
        string::WasmString,
        token::Token,
    },
    wrausmt_runtime::{
        runtime::values::{Num, Ref, Value},
        syntax::{
            self as modulesyntax,
            location::Location,
            types::{NumType, RefType},
            Id, Resolved, UncompiledExpr, Unvalidated,
        },
    },
};

pub trait SpecParser {
    fn parse_spec_test(&mut self) -> Result<SpecTestScript>;
    fn try_cmd_entry(&mut self) -> Result<Option<CmdEntry>>;
    fn try_cmd(&mut self) -> Result<Option<Cmd>>;
    fn try_module_cmd(&mut self) -> Result<Option<Cmd>>;
    fn expect_spec_module(&mut self) -> Result<Module>;
    fn try_spec_module(&mut self) -> Result<Option<Module>>;
    fn try_register_cmd(&mut self) -> Result<Option<Cmd>>;
    fn try_action_cmd(&mut self) -> Result<Option<Cmd>>;
    fn try_action(&mut self) -> Result<Option<Action>>;
    fn expect_action(&mut self) -> Result<Action>;
    fn try_assertion_cmd(&mut self) -> Result<Option<Cmd>>;
    fn try_invoke_action(&mut self) -> Result<Option<Action>>;
    fn try_get_action(&mut self) -> Result<Option<Action>>;
    fn try_meta_cmd(&mut self) -> Result<Option<Cmd>>;
    fn try_assert_return(&mut self) -> Result<Option<Assertion>>;
    fn try_assert_exhaustion(&mut self) -> Result<Option<Assertion>>;
    fn try_assert_trap(&mut self) -> Result<Option<Assertion>>;
    fn try_assert_invalid(&mut self) -> Result<Option<Assertion>>;
    fn try_assert_malformed(&mut self) -> Result<Option<Assertion>>;
    fn try_assert_unlinkable(&mut self) -> Result<Option<Assertion>>;
    fn try_result(&mut self) -> Result<Option<ActionResult>>;
    fn try_const(&mut self) -> Result<Option<Const>>;
    fn try_num_type(&mut self) -> Result<Option<NumType>>;
    fn try_nan_pat(&mut self, nt: NumType) -> Result<Option<NaNPat>>;
    fn expect_num(&mut self, nt: NumType) -> Result<Num>;
}

impl<R: Read> SpecParser for Parser<R> {
    fn parse_spec_test(&mut self) -> Result<SpecTestScript> {
        pctx!(self, "parse spec test");
        self.assure_started()?;
        match self.zero_or_more(Parser::try_cmd_entry) {
            Ok(cmds) => {
                if self.current.token != Token::Eof {
                    return Err(self.unexpected_token("cmd"));
                }
                Ok(SpecTestScript { cmds })
            }
            Err(e) => Err(e),
        }
    }

    fn try_cmd_entry(&mut self) -> Result<Option<CmdEntry>> {
        pctx!(self, "try cmd entry");
        let location = self.current.location;
        self.try_cmd()
            .map(|c| c.map(|cmd| CmdEntry { cmd, location }))
    }

    fn try_cmd(&mut self) -> Result<Option<Cmd>> {
        pctx!(self, "try cmd");
        self.first_of(&[
            Self::try_register_cmd,
            Self::try_action_cmd,
            Self::try_assertion_cmd,
            Self::try_meta_cmd,
            Self::try_module_cmd,
        ])
    }

    fn try_module_cmd(&mut self) -> Result<Option<Cmd>> {
        pctx!(self, "try module cmd");
        if let Some(module) = self.try_spec_module()? {
            return Ok(Some(Cmd::Module(module)));
        }

        Ok(None)
    }

    fn expect_spec_module(&mut self) -> Result<Module> {
        pctx!(self, "expect spec module");
        self.try_spec_module()?
            .ok_or(self.unexpected_token("spec module"))
    }

    fn try_spec_module(&mut self) -> Result<Option<Module>> {
        pctx!(self, "try spec module");
        if !self.try_expr_start("module")? {
            return if let Some(inline_module) = self.try_module()? {
                Ok(Some(Module::Module(inline_module)))
            } else {
                Ok(None)
            };
        }

        let modname = self.try_id()?;

        let kw = self.try_keyword()?;

        match kw {
            Some(k) if k.as_str() == "binary" => {
                let strings = self.zero_or_more(Self::try_wasm_string)?;
                self.expect_close()?;
                Ok(Some(Module::Binary(modname, strings)))
            }
            Some(k) if k.as_str() == "quote" => {
                let strings = self.zero_or_more(Self::try_wasm_string)?;
                self.expect_close()?;
                Ok(Some(Module::Quote(modname, strings)))
            }
            _ => {
                if let Some(module) = self.try_module_rest(modname, true)? {
                    Ok(Some(Module::Module(module)))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn try_register_cmd(&mut self) -> Result<Option<Cmd>> {
        pctx!(self, "try register cmd");
        if !self.try_expr_start("register")? {
            return Ok(None);
        }

        let modname = self.expect_string()?;

        let id = self.try_id()?;

        self.expect_close()?;

        Ok(Some(Cmd::Register { modname, id }))
    }

    fn try_action_cmd(&mut self) -> Result<Option<Cmd>> {
        pctx!(self, "try action cmd");
        self.try_action().map(|a| a.map(Cmd::Action))
    }

    fn try_action(&mut self) -> Result<Option<Action>> {
        pctx!(self, "try action");
        self.first_of(&[Self::try_invoke_action, Self::try_get_action])
    }

    fn expect_action(&mut self) -> Result<Action> {
        pctx!(self, "expect action");
        self.try_action()?
            .ok_or(self.unexpected_token("spec test action"))
    }

    fn try_invoke_action(&mut self) -> Result<Option<Action>> {
        pctx!(self, "try invoke action");
        if !self.try_expr_start("invoke")? {
            return Ok(None);
        }

        let modname = self.try_id()?;

        let name = self.expect_string()?;

        let params = self.zero_or_more(Self::try_const)?;

        self.expect_close()?;

        Ok(Some(Action::Invoke {
            modname,
            name,
            params,
        }))
    }

    fn try_get_action(&mut self) -> Result<Option<Action>> {
        pctx!(self, "try get action");
        if !self.try_expr_start("get")? {
            return Ok(None);
        }

        let modname = self.try_id()?;

        let name = self.expect_string()?;

        self.expect_close()?;

        Ok(Some(Action::Get { modname, name }))
    }

    fn try_assertion_cmd(&mut self) -> Result<Option<Cmd>> {
        pctx!(self, "try assertion cmd");
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
        pctx!(self, "try meta cmd");
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
        pctx!(self, "try assert return");
        if !self.try_expr_start("assert_return")? {
            return Ok(None);
        }

        let action = self.expect_action()?;

        let results = self.zero_or_more(Self::try_result)?;

        self.expect_close()?;

        Ok(Some(Assertion::Return { action, results }))
    }

    fn try_assert_exhaustion(&mut self) -> Result<Option<Assertion>> {
        pctx!(self, "try assert exhaustion");
        if !self.try_expr_start("assert_exhaustion")? {
            return Ok(None);
        }

        let action = self.expect_action()?;
        let failure = self.expect_string()?;
        self.expect_close()?;

        Ok(Some(Assertion::Exhaustion { action, failure }))
    }

    fn try_assert_trap(&mut self) -> Result<Option<Assertion>> {
        pctx!(self, "try assert trap");
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
        pctx!(self, "try assert invalid");
        if !self.try_expr_start("assert_invalid")? {
            return Ok(None);
        }

        let module = self.expect_spec_module()?;

        let failure = self.expect_string()?;

        self.expect_close()?;

        Ok(Some(Assertion::Invalid { module, failure }))
    }

    fn try_assert_malformed(&mut self) -> Result<Option<Assertion>> {
        pctx!(self, "try assert malformed");
        if !self.try_expr_start("assert_malformed")? {
            return Ok(None);
        }

        let module = self.expect_spec_module()?;

        let failure = self.expect_string()?;

        self.expect_close()?;

        Ok(Some(Assertion::Malformed { module, failure }))
    }

    fn try_assert_unlinkable(&mut self) -> Result<Option<Assertion>> {
        pctx!(self, "try assert unlinkable");
        if !self.try_expr_start("assert_unlinkable")? {
            return Ok(None);
        }

        let module = self.expect_spec_module()?;

        let failure = self.expect_string()?;

        self.expect_close()?;

        Ok(Some(Assertion::Unlinkable { module, failure }))
    }

    fn try_result(&mut self) -> Result<Option<ActionResult>> {
        pctx!(self, "try result");
        if self.try_expr_start("ref.null")? {
            let reftype = self.expect_heaptype()?;
            self.expect_close()?;
            let result = match reftype {
                RefType::Func => ActionResult::Func,
                RefType::Extern => ActionResult::Extern,
            };
            return Ok(Some(result));
        }

        if self.try_expr_start("ref.func")? {
            self.expect_close()?;
            return Ok(Some(ActionResult::Func));
        }

        if self.try_expr_start("ref.extern")? {
            let _reftype = self.expect_u32()?;
            self.expect_close()?;
            return Ok(Some(ActionResult::Extern));
        }

        let nt = self.try_num_type()?;

        match nt {
            Some(nt) => {
                if let Some(nanpat) = self.try_nan_pat(nt)? {
                    self.expect_close()?;
                    Ok(Some(ActionResult::NumPat(NumPat::NaNPat(nanpat))))
                } else {
                    let num = self.expect_num(nt)?;
                    self.expect_close()?;
                    Ok(Some(ActionResult::NumPat(NumPat::Num(num))))
                }
            }
            _ => Ok(None),
        }
    }

    fn try_const(&mut self) -> Result<Option<Const>> {
        pctx!(self, "try const");
        if self.try_expr_start("ref.null")? {
            let reftype = self.expect_heaptype()?;
            self.expect_close()?;
            return Ok(Some(Const::RefNull(reftype)));
        }

        if self.try_expr_start("ref.host")? {
            let val = self.expect_u32()?;
            self.expect_close()?;
            return Ok(Some(Const::RefHost(val)));
        }

        if self.try_expr_start("ref.extern")? {
            let val = self.expect_u32()?;
            self.expect_close()?;
            return Ok(Some(Const::RefHost(val)));
        }

        let nt = self.try_num_type()?;

        match nt {
            Some(nt) => {
                let num = self.expect_num(nt)?;
                self.expect_close()?;
                Ok(Some(Const::Num(num)))
            }
            _ => Ok(None),
        }
    }

    fn try_num_type(&mut self) -> Result<Option<NumType>> {
        pctx!(self, "try num type");
        let kw = self.peek_next_keyword()?.map(|kw| kw.as_str());
        let found = match kw {
            Some("i32.const") => Some(NumType::I32),
            Some("i64.const") => Some(NumType::I64),
            Some("f32.const") => Some(NumType::F32),
            Some("f64.const") => Some(NumType::F64),
            _ => None,
        };
        match found {
            Some(nt) => {
                self.advance()?;
                self.advance()?;
                Ok(Some(nt))
            }
            _ => Ok(None),
        }
    }

    fn try_nan_pat(&mut self, nt: NumType) -> Result<Option<NaNPat>> {
        pctx!(self, "try nan pat");
        let kw = self.peek_keyword()?.map(|kw| kw.as_str());
        match kw {
            Some("nan:canonical") => {
                self.advance()?;
                Ok(Some(NaNPat::Canonical(nt)))
            }
            Some("nan:arithmetic") => {
                self.advance()?;
                Ok(Some(NaNPat::Arithmetic(nt)))
            }
            _ => Ok(None),
        }
    }

    fn expect_num(&mut self, nt: NumType) -> Result<Num> {
        pctx!(self, "expect num");
        let result = match nt {
            NumType::I32 => Num::I32(self.expect_i32()? as u32),
            NumType::I64 => Num::I64(self.expect_i64()? as u64),
            NumType::F32 => Num::F32(self.expect_f32()?),
            NumType::F64 => Num::F64(self.expect_f64()?),
        };
        Ok(result)
    }
}

/// ```text
/// script: <cmd>*
/// ```
#[derive(Debug, Default)]
pub struct SpecTestScript {
    pub cmds: Vec<CmdEntry>,
}

/// ```text
/// cmd:
///   <module>                                   ;; define, validate, and initialize module
///   ( register <string> <name>? )              ;; register module for imports
///   <action>                                   ;; perform action and print results
///   <assertion>                                ;; assert result of an action
///   <meta>
/// ```
#[derive(Debug)]
pub enum Cmd {
    Module(Module),
    Register {
        modname: String,
        id:      Option<Id>,
    },
    Action(Action),
    Assertion(Assertion),
    Meta(Meta),
}

#[derive(Debug)]
pub struct CmdEntry {
    pub cmd:      Cmd,
    pub location: Location,
}

/// ```text
/// module:
///   ...
///   ( module <name>? binary <string>* )        ;; module in binary format (may be malformed)
///   ( module <name>? quote <string>* )         ;; module quoted in text (may be malformed)
/// ```
#[derive(Debug)]
pub enum Module {
    Module(modulesyntax::Module<Resolved, Unvalidated, UncompiledExpr<Resolved>>),
    Binary(Option<Id>, Vec<WasmString>),
    Quote(Option<Id>, Vec<WasmString>),
}

/// ```text
/// action:
///   ( invoke <name>? <string> <const>* )       ;; invoke function export
///   ( get <name>? <string> )                   ;; get global export
/// ```
#[derive(Debug)]
pub enum Action {
    Invoke {
        modname: Option<Id>,
        name:    String,
        params:  Vec<Const>,
    },
    Get {
        modname: Option<Id>,
        name:    String,
    },
}

impl Action {
    pub fn name(&self) -> &str {
        match self {
            Action::Invoke { name, .. } | Action::Get { name, .. } => name,
        }
    }
}

/// ```text
/// const:
///   ( <num_type>.const <num> )                 ;; number value
///   ( ref.null <ref_kind> )                    ;; null reference
///   ( ref.host <nat> )                         ;; host reference
/// ```
#[derive(Debug)]
pub enum Const {
    Num(Num),
    RefNull(RefType),
    RefHost(u32),
}

impl From<Const> for Value {
    fn from(c: Const) -> Value {
        match c {
            Const::Num(p) => p.into(),
            Const::RefHost(h) => Value::Ref(Ref::Extern(h)),
            Const::RefNull(t) => match t {
                RefType::Func => Value::Ref(Ref::Null(RefType::Func)),
                RefType::Extern => Value::Ref(Ref::Null(RefType::Extern)),
            },
        }
    }
}

#[rustfmt::skip]
/// ```text
/// assertion:
///   ( assert_return <action> <result>* )       ;; assert action has expected results
///   ( assert_trap <action> <failure> )         ;; assert action traps with given failure string
///   ( assert_exhaustion <action> <failure> )   ;; assert action exhausts system resources
///   ( assert_malformed <module> <failure> )    ;; assert module cannot be decoded with given failure string
///   ( assert_invalid <module> <failure> )      ;; assert module is invalid with given failure string
///   ( assert_unlinkable <module> <failure> )   ;; assert module fails to link
///   ( assert_trap <module> <failure> )         ;; assert module traps on instantiation
/// ```
#[derive(Debug)]
pub enum Assertion {
  Return {
    action:  Action,
    results: Vec<ActionResult>,
  },
  ActionTrap {
    action:  Action,
    failure: String,
  },
  Exhaustion {
    action:  Action,
    failure: String,
  },
  Malformed {
    module:  Module,
    failure: String,
  },
  Invalid {
    module:  Module,
    failure: String,
  },
  Unlinkable {
    module:  Module,
    failure: String,
  },
  ModuleTrap {
    module:  Module,
    failure: String,
  },
}

/// ```text
/// result:
///   ( <num_type>.const <num_pat> )
///   ( ref.extern )
///   ( ref.func )
/// ```
#[derive(Copy, Clone, Debug)]
pub enum ActionResult {
    NumPat(NumPat),
    Extern,
    Func,
}

/// ```text
/// num_pat:
///   <value>                                    ;; literal result
///   nan:canonical                              ;; NaN in canonical form
///   nan:arithmetic                             ;; NaN with 1 in MSB of payload
/// ```
#[derive(Copy, Clone, Debug)]
pub enum NumPat {
    Num(Num),
    NaNPat(NaNPat),
}

#[derive(Clone, Copy, Debug)]
pub enum NaNPat {
    Canonical(NumType),
    Arithmetic(NumType),
}

impl NaNPat {
    pub fn accepts(&self, n: Num) -> bool {
        match self {
            NaNPat::Canonical(nt) => match n {
                Num::F32(f) if &n.numtype() == nt => f.is_nan(),
                Num::F64(f) if &n.numtype() == nt => f.is_nan(),
                _ => false,
            },
            NaNPat::Arithmetic(nt) => match n {
                Num::F32(f) if &n.numtype() == nt => f.is_nan(),
                Num::F64(f) if &n.numtype() == nt => f.is_nan(),
                _ => false,
            },
        }
    }
}

/// ```text
/// meta:
///  ( script <name>? <script> )                ;; name a subscript
///  ( input <name>? <string> )                 ;; read script or module from file
///  ( output <name>? <string>? )               ;; output module to stout or file
/// ```
#[derive(Debug)]
pub enum Meta {
    Script { id: Option<Id>, script: String },
    Input { id: Option<Id>, file: String },
    Output { id: Option<Id>, file: String },
}
