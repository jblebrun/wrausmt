use std::fs::File;
use rust_wasm::format::binary::parse;
use rust_wasm::runtime::Runtime;
use rust_wasm::error::*;

#[test]
fn simplefunc() -> Result<()> {
    let mut runtime = Runtime::new();


    let mut f = File::open("testdata/simplefunc.wasm").unwrap();
    let test_mod = parse(&mut f)?;
    let mod2 = test_mod.clone();
    println!("MODULE {:x?}",test_mod);


    let mod_inst = runtime.load(test_mod);
    let res1 = runtime.call(mod_inst, "test", &[100]).unwrap();
    assert_eq!(res1, 142);
    
    let mod_inst2 = runtime.load(mod2);
    let res2 = runtime.call(mod_inst2, "test", &[4]).unwrap();
    assert_eq!(res2, 46);
    Ok(())
}
