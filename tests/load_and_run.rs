use std::fs::File;
use wrausmt::{format::binary::parse, runtime::values::Value};
use wrausmt::runtime::Runtime;
use wrausmt::error::{Result, ResultFrom};

#[test]
fn simplefunc() -> Result<()> {
    let mut runtime = Runtime::new();

    let mut f = File::open("testdata/simplefunc.wasm").wrap("load file")?;
    let test_mod = parse(&mut f).wrap("parse error")?;
    let mod2 = test_mod.clone();
    println!("MODULE {:x?}", test_mod);

    let mod_inst = runtime.load(test_mod)?;
    let res1 = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res1.first().unwrap();
    assert_eq!(v, 142u32.into());

    let mod_inst2 = runtime.load(mod2)?;
    let res2 = runtime.call(&mod_inst2, "test", &[4u32.into()])?;
    let v: Value = *res2.first().unwrap();
    assert_eq!(v, 46u32.into());
    Ok(())
}

#[test]
fn locals() -> Result<()> {
    let mut runtime = Runtime::new();

    let mut f = File::open("testdata/locals.wasm").wrap("load file")?;
    let test_mod = parse(&mut f).wrap("parse error")?;
    println!("MODULE {:x?}", test_mod);

    let mod_inst = runtime.load(test_mod)?;
    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, 694u32.into());

    Ok(())
}

#[test]
fn callandglobal() -> Result<()> {
    let mut runtime = Runtime::new();

    let mut f = File::open("testdata/callandglobal.wasm").wrap("load file")?;
    let test_mod = parse(&mut f).wrap("parse error")?;
    println!("MODULE {:x?}", test_mod);

    let mod_inst = runtime.load(test_mod)?;
    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, (100u32 + 100u32 + 0x77).into());

    Ok(())
}

#[test]
fn simplemem() -> Result<()> {
    let mut runtime = Runtime::new();

    let mut f = File::open("testdata/simplemem.wasm").wrap("load file")?;
    let test_mod = parse(&mut f).wrap("parse error")?;
    println!("MODULE {:x?}", test_mod);

    let mod_inst = runtime.load(test_mod)?;
    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, 101u32.into());
    
    runtime.call(&mod_inst, "inc", &[])?;

    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, 103u32.into());

    Ok(())
}
