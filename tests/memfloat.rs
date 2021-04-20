use std::fs::File;
use wrausmt::format::binary::parse;
use wrausmt::runtime::Runtime;
use wrausmt::error::{Result, ResultFrom};
use wrausmt::runner;

#[test]
fn memfloat() -> Result<()> {
    let runtime = &mut Runtime::new();

    let mut f = File::open("testdata/meminstr.wasm").wrap("load file")?;
    let test_mod = parse(&mut f).wrap("parse error")?;
    let mod_inst = runtime.load(test_mod)?;

    runner!(runtime, &mod_inst);

    exec_method!("put32_f", 8745897.5f32, 0)?;
    let res1 = exec_method!("get32_f", 0)?;
    assert_eq!(res1, (8745897.5f32).into());

    let c2: f64 = 897459874895.625;
    exec_method!("put64_f", c2, 0)?;
    let res1 = exec_method!("get64_f", 0)?;
    assert_eq!(res1, c2.into());
    
    Ok(())
}
