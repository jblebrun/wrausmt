mod runtime;
mod types;
mod instructions;
mod module;

use types::*;
use instructions::Inst;
use runtime::Runtime;
use module::*;

fn main() {
    let mut runtime = Runtime::new();

    let test_mod = Module {
        types: Box::new([
            FunctionType {
                params: Box::new([ValueType::Num(NumType::I32)]),
                result: Box::new([ValueType::Num(NumType::I32)]),
            }
        ]),
        funcs: Box::new([
            Function {
                functype: 0,
                body: Box::new([Inst::LocalGet(0), Inst::Const32(42), Inst::Add32])
            }
        ]),
        exports: Box::new([
            Export {
                name: "test".to_string(),
                idx: 0
            }
        ])
    };

    let mod_inst = runtime.load(test_mod);
    runtime.call(&mod_inst, "test");
}
