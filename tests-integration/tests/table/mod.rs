use {
    wrausmt_format::file_loader::FileLoader,
    wrausmt_runtime::runtime::{values::Value, Runtime},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn table() -> Result<()> {
    let mut runtime = Runtime::new();
    println!("CUR: {:?}", std::env::current_dir());
    let mod_inst = runtime.load_file("tests/table/data/table.wat")?;

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
