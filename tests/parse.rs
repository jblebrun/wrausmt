use wrausmt::format::text::Parser;
use wrausmt::format::text::token::Token;
use wrausmt::format::text::lex::Tokenizer;
use wrausmt::format::text::module::syntax::Field;
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
    for field in &module.fields {
        println!("Field {:#?}", field);
    }

    assert_eq!(module.fields[0], Field::Type(typefield! { "$void"; [] -> [] }));
    assert_eq!(module.fields[1], Field::Type(typefield! { None; [I32] -> [I32] }));
    assert_eq!(module.fields[2], Field::Type(typefield! { [I32 "$x"] -> [] }));
    assert_eq!(module.fields[3], Field::Type(typefield! { [Func] -> [] }));
    assert_eq!(module.fields[4], Field::Type(typefield! { [Extern] -> [] }));

    Ok(())
}
