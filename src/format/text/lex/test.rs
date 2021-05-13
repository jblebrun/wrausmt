use super::super::token::{FileToken, Sign, Token};
use super::Tokenizer;
use crate::format::text::{
    lex::error::{Result, WithContext},
    string::WasmString,
    token::{Base, NumToken},
};

macro_rules! expect_tokens {
    ( $to_parse:expr, $($t:expr),* ) => {
        let mut tokenizer = Tokenizer::new($to_parse.as_bytes())?;
        $(
            let rtok = tokenizer.next();
            match rtok {
                Some(Ok(FileToken { token, location: _ })) => assert_eq!(token, $t),
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
        Token::String(WasmString::from_bytes("hello".as_bytes()).unwrap()),
        Token::Whitespace,
        Token::Open,
        Token::Number(NumToken::Float(
            Sign::Unspecified,
            Base::Dec,
            "5".into(),
            "6".into(),
            "".into()
        )),
        Token::Whitespace,
        Token::Number(NumToken::Integer(Sign::Negative, Base::Hex, "F".into())),
        Token::Whitespace,
        Token::Number(NumToken::Integer(Sign::Unspecified, Base::Hex, "F".into())),
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
    let expect = "this is a 😃 string".as_bytes();
    expect_tokens!(
        "\"this is a 😃 string\"",
        Token::String(WasmString::from_bytes(expect).unwrap())
    );
    Ok(())
}

#[test]
fn bytes_string() -> Result<()> {
    let expect = b"this has raw data\x01\x02\xE4";
    expect_tokens!(
        "\"this has raw data\\01\\02\\E4\"",
        Token::String(WasmString::from_bytes(expect).unwrap())
    );
    Ok(())
}

fn inttoken(sign: char, base: Base, digits: &str) -> Token {
    Token::Number(NumToken::Integer(sign.into(), base, digits.into()))
}

fn floattoken(sign: char, base: Base, whole: &str, frac: &str, exp: &str) -> Token {
    Token::Number(NumToken::Float(
        sign.into(),
        base,
        whole.into(),
        frac.into(),
        exp.into(),
    ))
}

#[test]
fn bare_integer_dec() -> Result<()> {
    expect_tokens!("3452", inttoken(' ', Base::Dec, "3452"));
    expect_tokens!("0", inttoken(' ', Base::Dec, "0"));
    expect_tokens!("+3452", inttoken('+', Base::Dec, "3452"));
    expect_tokens!("-3452", inttoken('-', Base::Dec, "3452"));
    expect_tokens!("+0", inttoken('+', Base::Dec, "0"));
    expect_tokens!("-0", inttoken('-', Base::Dec, "0"));
    Ok(())
}

#[test]
fn bare_float_dec() -> Result<()> {
    expect_tokens!("1.5", floattoken(' ', Base::Dec, "1", "5", ""));
    expect_tokens!("-1.5", floattoken('-', Base::Dec, "1", "5", ""));
    expect_tokens!("1.", floattoken(' ', Base::Dec, "1", "", ""));
    expect_tokens!("-1.", floattoken('-', Base::Dec, "1", "", ""));
    expect_tokens!("5e5", floattoken(' ', Base::Dec, "5", "", "5"));
    expect_tokens!("-5e5", floattoken('-', Base::Dec, "5", "", "5"));
    expect_tokens!("2.5e5", floattoken(' ', Base::Dec, "2", "5", "5"));
    expect_tokens!("-2.5e5", floattoken('-', Base::Dec, "2", "5", "5"));
    Ok(())
}

#[test]
fn bare_integer_hex() -> Result<()> {
    expect_tokens!("0x60", inttoken(' ', Base::Hex, "60"));
    expect_tokens!("0xFF", inttoken(' ', Base::Hex, "FF"));
    expect_tokens!("-0xFF", inttoken('-', Base::Hex, "FF"));
    expect_tokens!("+0xFF", inttoken('+', Base::Hex, "FF"));
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
    expect_tokens!("nan", Token::Number(NumToken::NaN(Sign::Unspecified)));
    expect_tokens!("-nan", Token::Number(NumToken::NaN(Sign::Negative)));
    expect_tokens!("+nan", Token::Number(NumToken::NaN(Sign::Positive)));
    expect_tokens!(
        "nan:0x56",
        Token::Number(NumToken::NaNx(Sign::Unspecified, "56".into()))
    );
    expect_tokens!(
        "-nan:0x56",
        Token::Number(NumToken::NaNx(Sign::Negative, "56".into()))
    );
    expect_tokens!(
        "+nan:0x56",
        Token::Number(NumToken::NaNx(Sign::Positive, "56".into()))
    );
    Ok(())
}

#[test]
fn seps() -> Result<()> {
    expect_tokens!("100_000", inttoken(' ', Base::Dec, "100000"));
    expect_tokens!(
        "100_000.500_000_1",
        floattoken(' ', Base::Dec, "100000", "5000001", "")
    );
    Ok(())
}

#[test]
fn longer_test() -> Result<()> {
    let mut f = std::fs::File::open("testdata/locals.wat").ctx("opening file")?;
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
