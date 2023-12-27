use {
    format::loader::Loader,
    wrausmt_runtime::runtime::{values::Value, Runtime},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn importer() -> Result<()> {
    let mut runtime = Runtime::new();
    let importee = runtime.load_wast("data/importee.wat")?;
    runtime.register("src", importee);

    let importer = runtime.load_wast("data/importer.wat")?;
    let res1 = runtime.call(&importer, "test", &[100u32.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &125u32.into());

    Ok(())
}
