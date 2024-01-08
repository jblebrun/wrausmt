use {
    wrausmt_format::file_loader::FileLoader,
    wrausmt_runtime::runtime::{values::Value, Runtime},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn importer() -> Result<()> {
    let mut runtime = Runtime::new();
    let importee = runtime.load_file("tests/importing/data/importee.wat")?;
    runtime.register("src", importee);

    let importer = runtime.load_file("tests/importing/data/importer.wat")?;
    let res1 = runtime.call(&importer, "test", &[100u32.into()])?;
    let v1: &Value = res1.first().unwrap();
    assert_eq!(v1, &125u32.into());

    Ok(())
}
