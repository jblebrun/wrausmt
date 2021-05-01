use std::fs::File;
use wrausmt::format::binary::parse;
use wrausmt::runtime::Runtime;
use wrausmt::error::{Result, ResultFrom};
use wrausmt::runner;
use std::convert::TryInto;

#[test]
fn meminstr64_get() -> Result<()> {
    let runtime = &mut Runtime::new();

    let mut f = File::open("testdata/meminstr.wasm").wrap("load file")?;
    let test_mod = parse(&mut f).wrap("parse error")?;
    let mod_inst = runtime.load(test_mod)?;

    runner!(runtime, &mod_inst);

    exec_method!("put64", 0x873646368176F5F3u64, 0)?;
    
    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0x873646368176F5F3u64.into());

    let mut res1 = exec_method!("get64_8u", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0xF3u64.into());
    
    let mut res1 = exec_method!("get64_8s", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0xF3-0x100) as u64).into());

    let mut res1 = exec_method!("get64_16u", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0xF5F3u64.into());
    
    let mut res1 = exec_method!("get64_16s", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0xF5F3-0x10000) as u64).into());

    let mut res1 = exec_method!("get64_32u", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0x8176F5F3u64.into());
    
    let mut res1 = exec_method!("get64_32s", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0x8176F5F3i64-0x100000000i64) as u64).into());
    Ok(())
}

#[test]
fn meminstr64_put() -> Result<()> {
    let runtime = &mut Runtime::new();

    let mut f = File::open("testdata/meminstr.wasm").wrap("load file")?;
    let test_mod = parse(&mut f).wrap("parse error")?;
    let mod_inst = runtime.load(test_mod)?;

    runner!(runtime, &mod_inst);

    exec_method!("put64_8", 0x873646368176F5F3u64, 0)?;
    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0xF3u64).into());

    exec_method!("put64_16", 0x873646368176F5F3u64 ,0)?;
    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0xF5F3u64).into());

    exec_method!("put64_32", 0x873646368176F5F3u64 ,0)?;
    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0x8176F5F3u64).into());
    
    exec_method!("put64_8", 0x78u64, 0)?;
    exec_method!("put64_8", 0x56u64, 1)?;
    exec_method!("put64_8", 0x34u64, 2)?;
    exec_method!("put64_8", 0x12u64, 3)?;
    exec_method!("put64_8", 0xabu64, 4)?;
    exec_method!("put64_8", 0xcdu64, 5)?;
    exec_method!("put64_8", 0xefu64, 6)?;
    exec_method!("put64_8", 0x77u64, 7)?;
    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0x77efcdab12345678u64).into());
    Ok(())
}




