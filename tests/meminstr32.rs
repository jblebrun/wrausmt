use std::fs::File;
use wrausmt::format::binary::parse;
use wrausmt::runtime::Runtime;
use wrausmt::error::{Result, ResultFrom};
use wrausmt::runner;
use std::convert::TryInto;

#[test]
fn meminstr32_get() -> Result<()> {
    let runtime = &mut Runtime::new();

    let mut f = File::open("testdata/meminstr.wasm").wrap("load file")?;
    let test_mod = parse(&mut f).wrap("parse error")?;
    let mod_inst = runtime.load(test_mod)?;

    runner!(runtime, &mod_inst);

    exec_method!("put32", 0x8176F5F3u32, 0)?;
    let mut res1 = exec_method!("get32", 0)?;
    let v1: u32 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0x8176F5F3u32.into());

    let mut res1 = exec_method!("get32_8u", 0)?;
    let v1: u32 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0xF3u32.into());
    
    let mut res1 = exec_method!("get32_8s", 0)?;
    let v1: u32 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0xF3-0x100) as u32).into());

    let mut res1 = exec_method!("get32_16u", 0)?;
    let v1: i32 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0xF5F3.into());
    
    let mut res1 = exec_method!("get32_16s", 0)?;
    let v1: u32 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0xF5F3-0x10000) as u32).into());

    Ok(())
}

#[test]
fn meminstr32_put() -> Result<()> {
    let runtime = &mut Runtime::new();

    let mut f = File::open("testdata/meminstr.wasm").wrap("load file")?;
    let test_mod = parse(&mut f).wrap("parse error")?;
    let mod_inst = runtime.load(test_mod)?;

    runner!(runtime, &mod_inst);

    exec_method!("put32_8", 0x8176F5F3u32, 0)?;
    let mut res1 = exec_method!("get32", 0)?;
    let v1: u32 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0xF3u8).into());

    exec_method!("put32_16", 0x8176F5F3u32, 0)?;
    let mut res1 = exec_method!("get32", 0)?;
    let v1: u32 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0xF5F3u16).into());
    Ok(())
}
