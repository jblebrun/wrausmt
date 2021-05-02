use wrausmt::runtime::Runtime;
use wrausmt::runner;
use wrausmt::loader::Loader;
use std::convert::TryInto;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn memfloat() -> Result<()> {
    let runtime = &mut Runtime::new();
    let mod_inst = runtime.load_wasm("testdata/meminstr.wasm")?;

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
