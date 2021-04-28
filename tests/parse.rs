use wrausmt::format::text::Parser;
use wrausmt::format::text::token::Token;
use wrausmt::format::text::lex::Tokenizer;
use wrausmt::error::{Result, ResultFrom};
use wrausmt::typefield;

#[test]
fn basic_parse() -> Result<()> {
    let f = std::fs::File::open("testdata/locals.wat").wrap("opening file")?;

    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let module = parser.try_module()?.unwrap();

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
