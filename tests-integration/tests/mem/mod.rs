use {
    std::convert::TryInto,
    wrausmt_format::file_loader::FileLoader,
    wrausmt_runtime::{runner, runtime::Runtime},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn memfloat() -> Result<()> {
    let runtime = &mut Runtime::new();
    let mod_inst = runtime.load_file("tests/mem/data/meminstr.wasm")?;

    runner!(runtime, &mod_inst);

    exec_method!("put32_f", 0, 8745897.5f32)?;
    let mut res1 = exec_method!("get32_f", 0)?;
    let v1: f32 = res1.remove(0).try_into()?;
    assert_eq!(v1, 8745897.5f32);

    let c2: f64 = 897459874895.625;
    exec_method!("put64_f", 0, c2)?;
    let mut res1 = exec_method!("get64_f", 0)?;
    let v1: f64 = res1.remove(0).try_into()?;
    assert_eq!(v1, c2);

    Ok(())
}

#[test]
fn meminstr32_get() -> Result<()> {
    let runtime = &mut Runtime::new();
    let mod_inst = runtime.load_file("tests/mem/data/meminstr.wasm")?;

    runner!(runtime, &mod_inst);

    exec_method!("put32", 0, 0x8176F5F3u32)?;
    let mut res1 = exec_method!("get32", 0)?;
    let v1: u32 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0x8176F5F3u32);

    let mut res1 = exec_method!("get32_8u", 0)?;
    let v1: u32 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0xF3u32);

    let mut res1 = exec_method!("get32_8s", 0)?;
    let v1: u32 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0xF3 - 0x100) as u32));

    let mut res1 = exec_method!("get32_16u", 0)?;
    let v1: i32 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0xF5F3);

    let mut res1 = exec_method!("get32_16s", 0)?;
    let v1: u32 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0xF5F3 - 0x10000) as u32));

    Ok(())
}

#[test]
fn meminstr32_put() -> Result<()> {
    let runtime = &mut Runtime::new();
    let mod_inst = runtime.load_file("tests/mem/data/meminstr.wasm")?;

    runner!(runtime, &mod_inst);

    exec_method!("put32_8", 0, 0x8176F5F3u32)?;
    let mut res1 = exec_method!("get32", 0)?;
    let v1: u32 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0xF3u8).into());

    exec_method!("put32_16", 0, 0x8176F5F3u32)?;
    let mut res1 = exec_method!("get32", 0)?;
    let v1: u32 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0xF5F3u16).into());
    Ok(())
}

#[test]
fn meminstr64_get() -> Result<()> {
    let runtime = &mut Runtime::new();
    let mod_inst = runtime.load_file("tests/mem/data/meminstr.wasm")?;

    runner!(runtime, &mod_inst);

    exec_method!("put64", 0, 0x873646368176F5F3u64)?;

    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0x873646368176F5F3u64);

    let mut res1 = exec_method!("get64_8u", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0xF3u64);

    let mut res1 = exec_method!("get64_8s", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0xF3 - 0x100) as u64));

    let mut res1 = exec_method!("get64_16u", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0xF5F3u64);

    let mut res1 = exec_method!("get64_16s", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0xF5F3 - 0x10000) as u64));

    let mut res1 = exec_method!("get64_32u", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, 0x8176F5F3u64);

    let mut res1 = exec_method!("get64_32s", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, ((0x8176F5F3i64 - 0x100000000i64) as u64));
    Ok(())
}

#[test]
fn meminstr64_put() -> Result<()> {
    let runtime = &mut Runtime::new();
    let mod_inst = runtime.load_file("tests/mem/data/meminstr.wasm")?;

    runner!(runtime, &mod_inst);

    exec_method!("put64_8", 0, 0x873646368176F5F3u64)?;
    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0xF3u64));

    exec_method!("put64_16", 0, 0x873646368176F5F3u64)?;
    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0xF5F3u64));

    exec_method!("put64_32", 0, 0x873646368176F5F3u64)?;
    let mut res1 = exec_method!("get64", 0)?;
    let v1: u64 = res1.remove(0).try_into()?;
    assert_eq!(v1, (0x8176F5F3u64));

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
    assert_eq!(v1, (0x77efcdab12345678u64));
    Ok(())
}

#[test]
fn memoffset() -> Result<()> {
    let runtime = &mut Runtime::new();
    let mod_inst = runtime.load_file("tests/mem/data/meminstr.wasm")?;

    runner!(runtime, &mod_inst);

    for i in 0..10 {
        let v = 2.3f32 * i as f32;
        println!("WRITE {:?}", v);
        exec_method!("put32_f", i * 4, v)?;
    }
    for i in 0..10 {
        let v = 2.3f32 * i as f32;
        let res1 = exec_method!("get32_f", i * 4)?;
        println!("READ BACK {:?}", res1);
        let v1 = res1.first().unwrap();
        assert_eq!(v1, &v.into());
    }

    for i in 0..10 {
        let v = 2.3f64 * i as f64;
        exec_method!("put64_f", i * 8, v)?;
    }
    for i in 0..10 {
        let v = 2.3f64 * i as f64;
        let res1 = exec_method!("get64_f", i * 8)?;
        let v1 = res1.first().unwrap();
        assert_eq!(v1, &v.into());
    }

    Ok(())
}
