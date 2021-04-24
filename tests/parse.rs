use wrausmt::format::text::Parser;
use wrausmt::error::{Result, ResultFrom};

#[test]
fn basic_parse() -> Result<()> {
    let f = std::fs::File::open("testdata/locals.wat").wrap("opening file")?;

    let sections = Parser::parse(f)?;

    for section in sections {
        println!("SECTION: {:?}", section);
    }
    Ok(())
}
