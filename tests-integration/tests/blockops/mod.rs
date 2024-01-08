use {
    wrausmt_format::{file_loader::FileLoader, loader::Result},
    wrausmt_runtime::runtime::{values::Value, Runtime},
};

#[test]
fn blockbr() -> Result<()> {
    let mut runtime = Runtime::new();
    let mod_inst = runtime.load_file("tests/blockops/data/blockbr.wat")?;

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
    let mod_inst = runtime.load_file("tests/blockops/data/if.wat")?;

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
