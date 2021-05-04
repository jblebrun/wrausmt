use std::convert::TryInto;
use wrausmt::loader::Loader;
use wrausmt::runner;
use wrausmt::runtime::Runtime;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn meminstr64_get() -> Result<()> {
    let runtime = &mut Runtime::new();
    let mod_inst = runtime.load_wasm("testdata/meminstr.wasm")?;

    runner!(runtime, &mod_inst);

    exec_method!("put64", 0, 0x873646368176F5F3u64)?;

    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0x873646368176F5F3u64.into());

    let mut res1 = exec_method!("get64_8u", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0xF3u64.into());

    let mut res1 = exec_method!("get64_8s", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0xF3 - 0x100) as u64).into());

    let mut res1 = exec_method!("get64_16u", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0xF5F3u64.into());

    let mut res1 = exec_method!("get64_16s", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0xF5F3 - 0x10000) as u64).into());

    let mut res1 = exec_method!("get64_32u", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0x8176F5F3u64.into());

    let mut res1 = exec_method!("get64_32s", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0x8176F5F3i64 - 0x100000000i64) as u64).into());
    Ok(())
}

#[test]
fn meminstr64_put() -> Result<()> {
    let runtime = &mut Runtime::new();
    let mod_inst = runtime.load_wasm("testdata/meminstr.wasm")?;

    runner!(runtime, &mod_inst);

    exec_method!("put64_8", 0, 0x873646368176F5F3u64)?;
    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0xF3u64).into());

    exec_method!("put64_16", 0, 0x873646368176F5F3u64)?;
    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0xF5F3u64).into());

    exec_method!("put64_32", 0, 0x873646368176F5F3u64)?;
    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0x8176F5F3u64).into());

    exec_method!("put64_8", 0, 0x78u64)?;
    exec_method!("put64_8", 1, 0x56u64)?;
    exec_method!("put64_8", 2, 0x34u64)?;
    exec_method!("put64_8", 3, 0x12u64)?;
    exec_method!("put64_8", 4, 0xabu64)?;
    exec_method!("put64_8", 5, 0xcdu64)?;
    exec_method!("put64_8", 6, 0xefu64)?;
    exec_method!("put64_8", 7, 0x77u64)?;
    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0x77efcdab12345678u64).into());
    Ok(())
}
