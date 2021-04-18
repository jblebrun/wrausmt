#[cfg(test)] 
mod test {
    use crate::format::text::token::{Sign, Token};
    use crate::format::text::lex::num::maybe_number;


    macro_rules! check {
        ( $s:expr, $t:expr ) => {
            {
                println!("Checking {}", $s);
                let result = maybe_number(&$s.to_string()).unwrap();
                assert_eq!(result, $t);
            }
        };
        ( $s:expr ) => {
            {
                println!("Checking {}", $s);
                let result = maybe_number(&$s.to_string());
                assert!(result == None);
            }
        }
    }

    #[test]
    fn unsigned() {
        check!("12345", Token::Unsigned(12345));
        check!("0", Token::Unsigned(0));
        check!("0xFF", Token::Unsigned(255));
        check!("0xFFFFFFFFFFFFFFFF", Token::Unsigned(u64::MAX));
        check!("18446744073709551615", Token::Unsigned(u64::MAX));
    }

    #[test]
    fn signed() {
        check!("+12345", Token::Signed(12345));
        check!("-12345", Token::Signed(-12345));
        check!("+0", Token::Signed(0));
        check!("-0", Token::Signed(0));
        check!("+0xFF", Token::Signed(255));
        check!("-0xFF", Token::Signed(-255));

        check!("+0x7FFFFFFFFFFFFFFF", Token::Signed(i64::MAX));
        check!("+9223372036854775807", Token::Signed(i64::MAX));

        check!("-0x7FFFFFFFFFFFFFFF", Token::Signed(i64::MIN+1));
        check!("-9223372036854775807", Token::Signed(i64::MIN+1));

        check!("-0x8000000000000000", Token::Signed(i64::MIN));
        check!("-9223372036854775808", Token::Signed(i64::MIN));
    }

    #[test]
    fn float() {
        println!("PARSING: ");
        //let x: f64 ="5.6p10".parse().unwrap();
        //println!("RES: {}", x);
        check!("1.1", Token::Float(1.1));
        check!("+1.1", Token::Float(1.1));
        check!("-1.1", Token::Float(-1.1));
        check!("0.1", Token::Float(0.1));
        check!("+0.1", Token::Float(0.1));
        check!("-0.1", Token::Float(-0.1));
        check!("10.6e5", Token::Float(10.6e5));
        check!("10.6e-5", Token::Float(10.6e-5));
        check!("892734987398473897410.6e-5", Token::Float(892734987398473897410.6e-5));
        check!(
            "892734987398473897410.656756785675678e-5", 
            Token::Float(892734987398473897410.656756785675678e-5)
        );
        // check!("0xFF.FF", Token::Float(0.1));
        // check!("+0xFF.FF", Token::Float(0.1));
        // check!("-0.FF.FF", Token::Float(-0.1));
        // TODO - fix parsing very large float literals
    }

    #[test]
    fn not_numbers() {
        check!(".");
        check!(".1");
        check!("12345j");
        check!("12.345j");
        check!("0x12345Z");
    }

    #[test]
    fn infs() {
        check!("inf", Token::Inf(Sign::Unspecified));
        check!("+inf", Token::Inf(Sign::Positive));
        check!("-inf", Token::Inf(Sign::Negative));
    }

    #[test]
    fn nans() {
        check!("nan", Token::NaN(Sign::Unspecified));
        check!("+nan", Token::NaN(Sign::Positive));
        check!("-nan", Token::NaN(Sign::Negative));
        check!("nan:0x56", Token::NaNx(Sign::Unspecified, 0x56));
        check!("+nan:0x56", Token::NaNx(Sign::Positive, 0x56));
        check!("-nan:0x56", Token::NaNx(Sign::Negative, 0x56));
    }
}
