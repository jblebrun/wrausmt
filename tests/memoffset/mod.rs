use wrausmt::loader::Loader;
use wrausmt::runner;
use wrausmt::runtime::Runtime;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn memoffset() -> Result<()> {
    let runtime = &mut Runtime::new();
    let mod_inst = runtime.load_wasm("testdata/meminstr.wasm")?;

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
