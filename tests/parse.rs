use wrausmt::format::text::parse::Parser;
use wrausmt::format::text::token::Token;
use wrausmt::format::text::lex::Tokenizer;
use wrausmt::typefield;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn basic_parse() -> Result<()> {
    let f = std::fs::File::open("testdata/locals.wat")?;

    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let module = parser.parse_full_module()?;

    if parser.current.token != Token::Eof {
        panic!("Incomplete parse {:?} {:?}",parser.current, parser.next); 
    }

    println!("{:?}", module);

    assert_eq!(module.types.len(), 6);
    assert_eq!(module.types[0], typefield! { "$void"; [] -> [] });
    assert_eq!(module.types[1], typefield! { None; [I32] -> [I32] });
    assert_eq!(module.types[2], typefield! { [I32 "$x"] -> [] });
    assert_eq!(module.types[3], typefield! { [Func] -> [] });
    assert_eq!(module.types[4], typefield! { [Extern] -> [] });
    assert_eq!(module.types[5], typefield! { [] -> [I32] });

    Ok(())
}

#[test]
fn block_parse() -> Result<()> {
    let f = std::fs::File::open("testdata/plainblock.wat")?;

    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let module = parser.try_module()?.unwrap();

    if parser.current.token != Token::Eof {
        panic!("Incomplete parse {:?} {:?}",parser.current, parser.next); 
    }
    println!("{:?}", module);

    Ok(())
}

#[test]
fn folded_block_parse() -> Result<()> {
    let f = std::fs::File::open("testdata/foldedblock.wat")?;

    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let module = parser.try_module()?.unwrap();

    if parser.current.token != Token::Eof {
        panic!("Incomplete parse {:?} {:?}",parser.current, parser.next); 
    }
    println!("{:?}", module);

    Ok(())
}

