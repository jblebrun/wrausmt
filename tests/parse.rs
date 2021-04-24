use wrausmt::format::text::{Field, Parser, FParam, FResult};
use wrausmt::format::text::typefield::TypeField;
use wrausmt::error::{Result, ResultFrom};
use wrausmt::typefield;



#[test]
fn basic_parse() -> Result<()> {
    let f = std::fs::File::open("testdata/locals.wat").wrap("opening file")?;

    let sections = Parser::parse(f)?;

    for section in &sections {
        println!("SECTION: {:?}", section);
    }

    assert_eq!(sections[0], Field::Type(typefield! { "$void"; [] -> [] }));
    assert_eq!(sections[1], Field::Type(typefield! { None; [I32] -> [I32] }));
    assert_eq!(sections[2], Field::Type(typefield! { [I32 "$x"] -> [] }));
    assert_eq!(sections[3], Field::Type(typefield! { [Func] -> [] }));
    assert_eq!(sections[4], Field::Type(typefield! { [Extern] -> [] }));

    Ok(())
}
