use std::rc::Rc;

use super::format::{Action, ActionResult, SpecTestScript};
use crate::{err, error::Result, format::text::compile::compile, runtime::{Runtime, instance::ModuleInstance, values::{Num, Ref, Value}}, spec::format::{Assertion, Cmd, Module}};


fn handle_action(
    runtime: &mut Runtime,
    module_instance: &Option<Rc<ModuleInstance>>,
    action: Action
) -> Result<Vec<Value>> {
    let module_instance = match module_instance {
        Some(mi) => mi,
        None => return err!("action invoked with no module")
    };
    match action {
        Action::Invoke { id, name, params } => {
            println!("INVOKE ACTION {:?} {} {:?}", id, name, params);
            let values: Vec<Value> = params.into_iter().map(|p| p.into()).collect();
            runtime.call(&module_instance, &name, &values)
        },
        Action::Get { id, name } => {
            println!("GET ACTION {:?} {}", id, name);
            Ok(vec![Value::Num(Num::I32(0))])
        }
    }
}

pub fn verify_result(results: Vec<Value>, expects: Vec<ActionResult>) -> Result<()> {
    if results.len() != expects.len() {
        return err!("Expect {} results but got {}", expects.len(), results.len())
    }

    let mut expects = expects;
    for result in results {
        let expect = expects.pop().unwrap();
        match expect {
            ActionResult::Num(np) => {
                let expect:Value  = np.into();
                if result != expect {
                    return err!("Expected {:?}, got {:?}", expect, result)
                }
            },
            ActionResult::Func => {
                if !matches!(result, Value::Ref(Ref::Func(_))) {
                    return err!("Expected Func, got {:?}", result)
                }
            }
            ActionResult::Extern => {
                if !matches!(result, Value::Ref(Ref::Extern(_))) {
                    return err!("Expected Extern, got {:?}", result)
                }
            }
        }
    }
    println!("TEST PASSED!");
    Ok(())
}
pub fn run_spec_test(script: SpecTestScript) -> Result<()> {
    println!("PARSING SCRIPT: ");
    
    let mut runtime = Runtime::new();
    
    let mut module: Option<Rc<ModuleInstance>> = None;

    for cmd in script.cmds {
        match cmd {
            Cmd::Module(m) => match m {
                Module::Module(m) =>  {
                    println!("NORMAL MODULE");
                    let compiled = compile(m)?;
                    module = Some(runtime.load(compiled)?);
                },
                Module::Binary(_) => println!("BINARY MODULE ACTION"),
                Module::Quote(_) => println!("QUOTE MODULE ACTION"),
            }
            Cmd::Register{string, name} => println!("REGISTER {} {}", string, name),
            Cmd::Action(a) => { handle_action(&mut runtime, &module, a)?; },
            Cmd::Assertion(a) => match a {
                Assertion::Return{action, results} => {
                    let result = handle_action(&mut runtime, &module, action)?;
                    println!("\nTEST CASE:");
                    println!("  ASSERT RETURN EXPECTS {:?}", results);
                    println!("  ASSERT RETURN GOT {:?}", result);
                    verify_result(result, results)?;
                    println!("\n\n");
                }
                _ => println!("ASSERT OTHER")
            }
            Cmd::Meta(m) => println!("META {:?}", m),
        }
    }
    Ok(())
}
