use {
    std::fs::File,
    wrausmt::{
        format::text::{
            lex::Tokenizer,
            parse::error::KindResult,
            parse_wast_data,
            token::{NumToken, Token},
        },
        loader::Result as LoadResult,
        syntax::{Module, Resolved},
        typefield,
    },
};

fn load_ast(filename: &str) -> LoadResult<Module<Resolved>> {
    let loaded = parse_wast_data(&mut File::open(filename)?)?;
    Ok(loaded)
}

#[test]
fn basic_parse() -> LoadResult<()> {
    let module = load_ast("testdata/locals.wat")?;

    println!("{:?}", module);

    assert_eq!(module.types.len(), 7);
    assert_eq!(module.types[0], typefield! { "$void"; [] -> [] });
    assert_eq!(module.types[1], typefield! { None; [I32] -> [I32] });
    assert_eq!(module.types[2], typefield! { [I32 "$x"] -> [] });
    assert_eq!(module.types[3], typefield! { [Func] -> [] });
    assert_eq!(module.types[4], typefield! { [Extern] -> [] });
    assert_eq!(module.types[5], typefield! { "$void2"; [F32] -> [F32] });
    assert_eq!(module.types[6], typefield! { [] -> [I32] });

    Ok(())
}

#[test]
fn block_parse() -> LoadResult<()> {
    let module = load_ast("testdata/plainblock.wat")?;
    println!("{:?}", module);
    Ok(())
}

#[test]
fn folded_block_parse() -> LoadResult<()> {
    let module = load_ast("testdata/foldedblock.wat")?;
    println!("{:?}", module);
    Ok(())
}

#[test]
fn table_parse() -> LoadResult<()> {
    let module = load_ast("testdata/table.wat")?;
    println!("{:?}", module);
    Ok(())
}

fn parse_numtoken(src: &str) -> KindResult<NumToken> {
    let mut tokenizer = Tokenizer::new(src.as_bytes())?;
    match tokenizer.next().unwrap()?.token {
        Token::Number(nt) => Ok(nt),
        t => panic!("didn't get num token {:?}", t),
    }
}

#[test]
fn parse_number() -> KindResult<()> {
    let tok = parse_numtoken("-600")?;
    assert_eq!(tok.as_i32()?, -600);
    assert_eq!(tok.as_i64()?, -600);
    assert_eq!(tok.as_f32()?, -600f32);
    assert_eq!(tok.as_f64()?, -600f64);

    let tok = parse_numtoken("601")?;
    assert_eq!(tok.as_i32()?, 601);
    assert_eq!(tok.as_i64()?, 601);
    assert_eq!(tok.as_f32()?, 601f32);
    assert_eq!(tok.as_f64()?, 601f64);

    let tok = parse_numtoken("0xFFFFFFFF")?;
    assert_eq!(tok.as_i32()?, -1);
    assert_eq!(tok.as_u32()?, 0xFFFFFFFF);
    assert_eq!(tok.as_i64()?, 0xFFFFFFFF);

    let tok = parse_numtoken("67.45")?;
    assert_eq!(tok.as_f32()?, 67.45);
    assert_eq!(tok.as_f64()?, 67.45);

    let tok = parse_numtoken("67.45e22")?;
    assert_eq!(tok.as_f32()?, 67.45e22);
    assert_eq!(tok.as_f64()?, 67.45e22f64);

    let tok = parse_numtoken("67.45e220")?;
    assert_eq!(tok.as_f64()?, 67.45e220f64);

    let tok = parse_numtoken("nan")?;
    assert!(tok.as_f32()?.is_nan());

    Ok(())
}
