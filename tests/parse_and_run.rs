use wrausmt::runtime::values::Value;
use wrausmt::runtime::Runtime;
use wrausmt::loader::Loader;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn simplefunc() -> Result<()> {
    let mut runtime = Runtime::new();
    let mod_inst = runtime.load_wast("testdata/simplefunc.wat")?;

    let res1 = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &142u32.into());

    Ok(())
}

#[test]
fn locals() -> Result<()> {
    let mut runtime = Runtime::new();
    let mod_inst = runtime.load_wast("testdata/locals.wat")?;

    let res1 = runtime.call(&mod_inst, "test", &[105u32.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &699u32.into());

    Ok(())
}

#[test]
fn blockbr() -> Result<()> {
    let mut runtime = Runtime::new();
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
    Ok(())
}
