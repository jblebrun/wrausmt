mod runtime;
mod types;
mod instructions;
mod module;
mod format;

use types::*;
use instructions::*;
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
        imports: Box::new([]),
        funcs: Box::new([
            Function {
                functype: 0,
                locals: Box::new([]),
                body: Box::new([LocalGet, 0x00, 0, 0 , 0, I32Const, 0x42, 0, 0, 0, I32Add, End])
            }
        ]),
        exports: Box::new([
            Export {
                name: "test".to_string(),
                idx: 0
            }
        ])
    };
    let mod2 = test_mod.clone();

    let mod_inst = runtime.load(test_mod);
    let mod_inst2 = runtime.load(mod2);
    runtime.call(mod_inst, "test", 100);
    runtime.call(mod_inst2, "test", 4);
}
