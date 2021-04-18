use super::super::token::{FileToken, Token, Sign};
use super::Tokenizer;
use crate::error::{Result, ResultFrom};

macro_rules! expect_tokens {
    ( $to_parse:expr, $($t:expr),* ) => {
        let mut tokenizer = Tokenizer::new($to_parse.as_bytes())?;
        $(
            let rtok = tokenizer.next();
            match rtok {
                Some(Ok(FileToken { token: tok, context: _ })) => assert_eq!(tok, $t),
                Some(Err(e)) => panic!("expected token, get {:?}", e),
                None => panic!("expected token, got eof")
            }
        )*
        let end = tokenizer.next();
        assert!(end.is_none(), "Expected none, got {:?}", end);
    }
}
#[test]
fn simple_parse() -> Result<()> {
    expect_tokens!(
        "(foo) \"hello\" (5.6 -0xF 0xF) (; comment (; nested ;) more ;)\n(yay)",
        Token::Open,
        Token::Keyword("foo".into()),
        Token::Close,
        Token::Whitespace,
        Token::String("hello".into()),
        Token::Whitespace,
        Token::Open,
        Token::Float(5.6),
        Token::Whitespace,
        Token::Signed(-15),
        Token::Whitespace,
        Token::Unsigned(15),
        Token::Close,
        Token::Whitespace,
        Token::BlockComment,
        Token::Whitespace,
        Token::Open,
        Token::Keyword("yay".into()),
        Token::Close
    );
    Ok(())
}

#[test]
fn bare_string() -> Result<()> {
    expect_tokens!(
        "\"this is a 😃 string\"",
        Token::String("this is a 😃 string".into())
    );
    Ok(())
}

#[test]
fn bare_integer_dec() -> Result<()> {
    expect_tokens!("3452", Token::Unsigned(3452));
    expect_tokens!("0", Token::Unsigned(0));
    expect_tokens!("+3452", Token::Signed(3452));
    expect_tokens!("-3452", Token::Signed(-3452));
    expect_tokens!("+0", Token::Signed(0));
    expect_tokens!("-0", Token::Signed(0));
    Ok(())
}

#[test]
fn bare_float_dec() -> Result<()> {
    expect_tokens!("1.5", Token::Float(1.5));
    expect_tokens!("-1.5", Token::Float(-1.5));
    expect_tokens!("1.", Token::Float(1.));
    expect_tokens!("-1.", Token::Float(-1.));
    expect_tokens!("5e5", Token::Float(500000.0));
    expect_tokens!("-5e5", Token::Float(-500000.0));
    expect_tokens!("2.5e6", Token::Float(2500000.0));
    expect_tokens!("-2.5e6", Token::Float(-2500000.0));
    Ok(())
}

#[test]
fn bare_integer_hex() -> Result<()> {
    expect_tokens!("0x60", Token::Unsigned(0x60));
    expect_tokens!("0xFF", Token::Unsigned(0xFF));
    expect_tokens!("-0xFF", Token::Signed(-0xFF));
    expect_tokens!("+0xFF", Token::Signed(0xFF));
    Ok(())
}

#[test]
fn reserved() -> Result<()> {
    expect_tokens!("0x60z", Token::Reserved("0x60z".into()));
    expect_tokens!("1.1asdf", Token::Reserved("1.1asdf".into()));
    expect_tokens!("+f", Token::Reserved("+f".into()));
    Ok(())
}

#[test]
fn nans() -> Result<()> {
    expect_tokens!("nan", Token::NaN(Sign::Unspecified));
    expect_tokens!("-nan", Token::NaN(Sign::Negative));
    expect_tokens!("+nan", Token::NaN(Sign::Positive));
    expect_tokens!("nan:0x56", Token::NaNx(Sign::Unspecified, 0x56));
    expect_tokens!("-nan:0x56", Token::NaNx(Sign::Negative, 0x56));
    expect_tokens!("+nan:0x56", Token::NaNx(Sign::Positive, 0x56));
    Ok(())
}

#[test]
fn seps() -> Result<()> {
    expect_tokens!("100_000", Token::Unsigned(100000));
    expect_tokens!("100_000.500_000_1", Token::Float(100000.5000001));
    Ok(())
}

#[test]
fn longer_test() -> Result<()> {
    let mut f = std::fs::File::open("testdata/locals.wat").wrap("opening file")?;
    let tokenizer = Tokenizer::new(&mut f)?;

    println!("here we go");
    for token in tokenizer {
        match token {
            Ok(t) => println!("{:?}", t),
            Err(e) => {
                println!("ERR {:?}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}
