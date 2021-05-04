use wrausmt::loader::load_ast;
use wrausmt::typefield;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn basic_parse() -> Result<()> {
    let module = load_ast("testdata/locals.wat")?;

    println!("{:?}", module);

    assert_eq!(module.types.len(), 7);
    assert_eq!(module.types[0], typefield! { "$void"; [] -> [] });
    assert_eq!(module.types[1], typefield! { None; [I32] -> [I32] });
    assert_eq!(module.types[2], typefield! { [I32 "$x"] -> [] });
    assert_eq!(module.types[3], typefield! { [Func] -> [] });
    assert_eq!(module.types[4], typefield! { [Extern] -> [] });
    assert_eq!(module.types[5], typefield! { [] -> [I32] });
    assert_eq!(module.types[6], typefield! { "$void2"; [F32] -> [F32] });

    Ok(())
}

#[test]
fn block_parse() -> Result<()> {
    let module = load_ast("testdata/plainblock.wat")?;
    println!("{:?}", module);
    Ok(())
}

#[test]
fn folded_block_parse() -> Result<()> {
    let module = load_ast("testdata/foldedblock.wat")?;
    println!("{:?}", module);
    Ok(())
}
