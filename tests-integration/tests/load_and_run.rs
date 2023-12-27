use {
    format::loader::Loader,
    wrausmt::runtime::{values::Value, Runtime},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn simplefunc() -> Result<()> {
    let mut runtime = Runtime::new();
    let env_mod = runtime.load_wasm("data/env.wasm")?;
    runtime.register("env", env_mod);
    let mod_inst = runtime.load_wasm("data/simplefunc.wasm")?;

    let res1 = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res1.first().unwrap();
    assert_eq!(v, 142u32.into());

    Ok(())
}

#[test]
fn locals() -> Result<()> {
    let mut runtime = Runtime::new();
    let env_mod = runtime.load_wasm("data/env.wasm")?;
    runtime.register("env", env_mod);
    let mod_inst = runtime.load_wasm("data/locals.wasm")?;

    println!("BEGIN TEST");
    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, 694u32.into());

    Ok(())
}

#[test]
fn callandglobal() -> Result<()> {
    let mut runtime = Runtime::new();
    let env_mod = runtime.load_wasm("data/env.wasm")?;
    runtime.register("env", env_mod);
    let mod_inst = runtime.load_wasm("data/callandglobal.wasm")?;

    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, (100u32 + 100u32 + 0x77).into());

    Ok(())
}

#[test]
fn simplemem() -> Result<()> {
    let mut runtime = Runtime::new();
    let env_mod = runtime.load_wasm("data/env.wasm")?;
    runtime.register("env", env_mod);
    let mod_inst = runtime.load_wasm("data/simplemem.wasm")?;

    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, 101u32.into());

    runtime.call(&mod_inst, "inc", &[])?;

    let res = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    let v: Value = *res.first().unwrap();
    assert_eq!(v, 103u32.into());

    Ok(())
}
