use super::lex::Tokenizer;
use super::token::Token;
use crate::error::{Result, ResultFrom};

macro_rules! expect_tokens {
    ( $to_parse:expr, $($t:expr),* ) => {
        let mut tokenizer = Tokenizer::new($to_parse.as_bytes())?;
        $(
            let tok = tokenizer.next().unwrap()?;
            assert_eq!(tok, $t);
            println!("OK: {:?}", tok);
        )*
        let end = tokenizer.next();
        assert!(end.is_none(), format!("Expected none, got {:?}", end));
    }
}

#[cfg(test)]
fn printout<T : AsRef<[u8]>>(to_parse: T) -> Result<()> {
    let tokenizer = Tokenizer::new(to_parse.as_ref())?;

    for token in tokenizer {
        println!("{:?}", token.unwrap());
    }

    Ok(())
}

#[test]
fn simple_parse() -> Result<()> {
    expect_tokens!(
        "(foo) \"hello\" (5.6 -0xF 0xF) (; comment (; nested ;) more ;)\n(yay)"
        ,
        Token::Open,
        Token::Keyword("foo".to_string()),
        Token::Close,
        Token::Whitespace,
        Token::String("hello".to_string()),
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
        Token::Keyword("yay".to_string()),
        Token::Close
    );
    Ok(())
}

#[test]
fn bare_string() -> Result<()> {
    printout("\"this is a string\"")
}

#[test]
fn bare_unsigned() -> Result<()> {
    printout("3452")?;
    printout("0")?;
    Ok(())
}

#[test]
fn bare_signed() -> Result<()> {
    printout("+3452")?;
    printout("-3452")?;
    printout("+0")?;
    printout("-0")?;
    Ok(())
}

#[test]
fn bare_float() -> Result<()> {
    printout("1.5")?;
    printout("-1.5")?;
    printout("1.")?;
    printout("-1.")?;
    printout("-5e10")?;
    printout("-5.6e10")?;
    printout("-2.5e2")?;
    printout("-2.5e-2")?;
    Ok(())
}

#[test]
fn bare_unsigned_hex() -> Result<()> {
    printout("0x60")?;
    printout("0xFF")?;
    printout("-0xFF")?;
    printout("+0xFF")?;
    printout("+0xFF.8p2")?;
    printout("-0xFF.7p-2")?;
    printout("-0xF")?;
    Ok(())
}

#[test]
fn nans() -> Result<()> {
    printout("nan")?;
    printout("inf")?;
    printout("nan:0x56")?;
    Ok(())
}

#[test]
fn seps() -> Result<()> {
    printout("100_000")?;
    printout("1.500_000_1")?;
    Ok(())
}

#[test]
fn longer_test() -> Result<()> {
    let mut f = std::fs::File::open("testdata/locals.wat").wrap("opening file")?;
    let mut tokenizer = Tokenizer::new(&mut f)?;

    println!("here we go");
    for token in tokenizer {
        match token {
            Ok(t) => println!("{:?}", t),
            Err(e) => {
                println!("ERR {:?}", e);
                return Err(e)
            }
        }
    }

    Ok(())
}
