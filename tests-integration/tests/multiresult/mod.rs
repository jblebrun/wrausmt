use {
    wrausmt_format::file_loader::FileLoader,
    wrausmt_runtime::runtime::{values::Value, Runtime},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn multiresult() -> Result<()> {
    let mut runtime = Runtime::new();
    let mod_inst = runtime.load_file("tests/multiresult/data/multiresult.wat")?;

    let res1 = runtime.call(&mod_inst, "test", &[])?;
    let v1: &Value = res1.first().unwrap();
    let v2: &Value = res1.get(1).unwrap();
    assert_eq!(v1, &0x42.into());
    assert_eq!(v2, &0xFF42u64.into());

    Ok(())
}
