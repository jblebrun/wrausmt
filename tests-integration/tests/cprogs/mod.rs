use {
    wrausmt_format::{file_loader::FileLoader, loader::Result},
    wrausmt_runtime::runtime::{values::Value, Runtime},
};

trait LoadEnv {
    fn load_env(&mut self) -> Result<()>;
}

impl LoadEnv for Runtime {
    fn load_env(&mut self) -> Result<()> {
        let module = self.load_file("tests/cprogs/data/env.wasm")?;
        self.register("env", module);
        Ok(())
    }
}

#[test]
fn callandglobal_wat() -> Result<()> {
    let mut runtime = Runtime::new();
    runtime.load_env()?;
    let mod_inst = runtime.load_file("tests/cprogs/data/callandglobal.wat")?;

    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, (100u32 + 100u32 + 0x77).into());

    Ok(())
}

#[test]
fn callandglobal_wasm() -> Result<()> {
    let mut runtime = Runtime::new();
    runtime.load_env()?;
    let mod_inst = runtime.load_file("tests/cprogs/data/callandglobal.wasm")?;

    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, (100u32 + 100u32 + 0x77).into());

    Ok(())
}

#[test]
fn locals_wat() -> Result<()> {
    let mut runtime = Runtime::new();
    runtime.load_env()?;
    let mod_inst = runtime.load_file("tests/cprogs/data/locals.wat")?;

    let res1 = runtime.call(&mod_inst, "test", &[105u32.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &699u32.into());

    Ok(())
}

#[test]
fn locals_wasm() -> Result<()> {
    let mut runtime = Runtime::new();
    runtime.load_env()?;
    let mod_inst = runtime.load_file("tests/cprogs/data/locals.wasm")?;

    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, 694u32.into());

    Ok(())
}
#[test]
fn simplefunc_wat() -> Result<()> {
    let mut runtime = Runtime::new();
    runtime.load_env()?;
    let mod_inst = runtime.load_file("tests/cprogs/data/simplefunc.wat")?;

    let res1 = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &142u32.into());

    Ok(())
}

#[test]
fn simplefunc_wasm() -> Result<()> {
    let mut runtime = Runtime::new();
    runtime.load_env()?;
    let mod_inst = runtime.load_file("tests/cprogs/data/simplefunc.wasm")?;

    let res1 = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res1.first().unwrap();
    assert_eq!(v, 142u32.into());

    Ok(())
}
#[test]
fn simplemem_wat() -> Result<()> {
    let mut runtime = Runtime::new();
    runtime.load_env()?;
    let mod_inst = runtime.load_file("tests/cprogs/data/simplemem.wat")?;

    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, 101u32.into());

    runtime.call(&mod_inst, "inc", &[])?;

    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, 103u32.into());

    Ok(())
}

#[test]
fn simplemem_wasm() -> Result<()> {
    let mut runtime = Runtime::new();
    runtime.load_env()?;
    let mod_inst = runtime.load_file("tests/cprogs/data/simplemem.wasm")?;

    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, 101u32.into());

    runtime.call(&mod_inst, "inc", &[])?;

    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, 103u32.into());

    Ok(())
}

#[test]
fn multi_module() -> Result<()> {
    let mut runtime = Runtime::new();
    runtime.load_env()?;
    let _ = runtime.load_file("tests/cprogs/data/simplefunc.wat")?;
    let mod_inst = runtime.load_file("tests/cprogs/data/locals.wat")?;

    let res1 = runtime.call(&mod_inst, "test", &[105u32.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &699u32.into());

    Ok(())
}
