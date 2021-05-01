use std::fs::File;
use wrausmt::format::binary::parse;
use wrausmt::runtime::Runtime;
use wrausmt::error::{Result, ResultFrom};
use wrausmt::runner;

#[test]
fn memoffset() -> Result<()> {
    let runtime = &mut Runtime::new();

    let mut f = File::open("testdata/meminstr.wasm").wrap("load file")?;
    let test_mod = parse(&mut f).wrap("parse error")?;
    let mod_inst = runtime.load(test_mod)?;

    runner!(runtime, &mod_inst);

    for i in 0..10 {
        let v = 2.3f32 * i as f32;
        println!("WRITE {:?}", v);
        exec_method!("put32_f", v, i*4)?;
    }
    for i in 0..10 {
        let v = 2.3f32 * i as f32;
        let res1 = exec_method!("get32_f", i*4)?;
        println!("READ BACK {:?}", res1);
        let v1 = res1.first().unwrap();
        assert_eq!(v1, &v.into());
    }

    for i in 0..10 {
        let v = 2.3f64 * i as f64;
        exec_method!("put64_f", v, i*8)?;
    }
    for i in 0..10 {
        let v = 2.3f64 * i as f64;
        let res1 = exec_method!("get64_f", i*8)?;
        let v1 = res1.first().unwrap();
        assert_eq!(v1, &v.into());
    }

    Ok(())
}
