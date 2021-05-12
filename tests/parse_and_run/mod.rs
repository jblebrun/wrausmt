use wrausmt::loader::Loader;
use wrausmt::runtime::values::Value;
use wrausmt::runtime::Runtime;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn simplefunc() -> Result<()> {
    let mut runtime = Runtime::new();
    let env_mod = runtime.load_wast("testdata/env.wat")?;
    runtime.register("env", env_mod);
    let mod_inst = runtime.load_wast("testdata/simplefunc.wat")?;

    let res1 = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &142u32.into());

    Ok(())
}

#[test]
fn locals() -> Result<()> {
    let mut runtime = Runtime::new();
    let env_mod = runtime.load_wast("testdata/env.wat")?;
    runtime.register("env", env_mod);
    let mod_inst = runtime.load_wast("testdata/locals.wat")?;

    let res1 = runtime.call(&mod_inst, "test", &[105u32.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &699u32.into());

    Ok(())
}

#[test]
fn multi_module() -> Result<()> {
    let mut runtime = Runtime::new();
    let env_mod = runtime.load_wast("testdata/env.wat")?;
    runtime.register("env", env_mod);
    let _ = runtime.load_wast("testdata/simplefunc.wat")?;

    let mod_inst = runtime.load_wast("testdata/locals.wat")?;

    let res1 = runtime.call(&mod_inst, "test", &[105u32.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &699u32.into());

    Ok(())
}

#[test]
fn blockbr() -> Result<()> {
    let mut runtime = Runtime::new();
    let env_mod = runtime.load_wast("testdata/env.wat")?;
    runtime.register("env", env_mod);
    let mod_inst = runtime.load_wast("testdata/blockbr.wat")?;

    let res1 = runtime.call(&mod_inst, "simpleblock", &[])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &14.into());

    let res1 = runtime.call(&mod_inst, "nestedblock", &[])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &13.into());

    let res1 = runtime.call(&mod_inst, "simpleblockplain", &[])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &114.into());

    let res1 = runtime.call(&mod_inst, "nestedblockplain", &[])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &113.into());

    let res1 = runtime.call(&mod_inst, "nestedbreakcleanup", &[])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &34.into());

    let res1 = runtime.call(&mod_inst, "nestedbreakcleanupparams", &[])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &34.into());
    Ok(())
}

#[test]
fn ifs() -> Result<()> {
    let mut runtime = Runtime::new();
    let env_mod = runtime.load_wast("testdata/env.wat")?;
    runtime.register("env", env_mod);
    let mod_inst = runtime.load_wast("testdata/if.wat")?;

    let res1 = runtime.call(&mod_inst, "simpleif", &[0.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x43.into());

    let res1 = runtime.call(&mod_inst, "simpleif", &[1.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x42.into());

    let res1 = runtime.call(&mod_inst, "nestedif", &[0.into(), 0.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x143.into());

    let res1 = runtime.call(&mod_inst, "nestedif", &[0.into(), 1.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x142.into());

    let res1 = runtime.call(&mod_inst, "nestedif", &[1.into(), 0.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x43.into());

    let res1 = runtime.call(&mod_inst, "nestedif", &[1.into(), 1.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x42.into());

    let res1 = runtime.call(&mod_inst, "nestedblockif", &[0.into(), 0.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x143.into());

    let res1 = runtime.call(&mod_inst, "nestedblockif", &[0.into(), 1.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x142.into());

    let res1 = runtime.call(&mod_inst, "nestedblockif", &[1.into(), 0.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x43.into());

    let res1 = runtime.call(&mod_inst, "nestedblockif", &[1.into(), 1.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x42.into());
    Ok(())
}

#[test]
fn table() -> Result<()> {
    let mut runtime = Runtime::new();
    let env_mod = runtime.load_wast("testdata/env.wat")?;
    runtime.register("env", env_mod);
    let mod_inst = runtime.load_wast("testdata/table.wat")?;

    let res1 = runtime.call(&mod_inst, "test", &[0.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x42.into());

    let res1 = runtime.call(&mod_inst, "test", &[1.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x43.into());

    let res1 = runtime.call(&mod_inst, "test", &[2.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &0x44.into());
    Ok(())
}

#[test]
fn multiresult() -> Result<()> {
    let mut runtime = Runtime::new();
    let env_mod = runtime.load_wast("testdata/env.wat")?;
    runtime.register("env", env_mod);
    let mod_inst = runtime.load_wast("testdata/multiresult.wat")?;

    let res1 = runtime.call(&mod_inst, "test", &[])?;
    let v1: &Value = res1.first().unwrap();
    let v2: &Value = res1.get(1).unwrap();
    assert_eq!(v1, &0x42.into());
    assert_eq!(v2, &0xFF42u64.into());

    Ok(())
}
