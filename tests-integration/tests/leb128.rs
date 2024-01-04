use {
    std::io::ErrorKind,
    wrausmt_format::binary::leb128::{LEB128Error, ReadLeb128},
};

macro_rules! assert_err {
    ( $res:expr, $match:pat, $cond:expr) => {
        match $res {
            Err($match) if $cond => Ok(()),
            Err(e) => Err(format!(
                "Expected error containg {}, got {}",
                stringify!($match),
                e
            )),
            Ok(_) => panic!("Expected error containg {}, got Ok", stringify!($match)),
        }
    };
    ( $res:expr, $match:pat) => {
        assert_err!($res, $match, true)
    };
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_leb128_u32() -> Result<()> {
    let data = vec![];
    let res = data.as_slice().read_u32_leb_128();
    assert_err!(
        res,
        LEB128Error::IOError(e),
        e.kind() == ErrorKind::UnexpectedEof
    )?;

    let data: Vec<u8> = vec![8];
    let res = data.as_slice().read_u32_leb_128().unwrap();
    assert_eq!(res, 8);

    let data: Vec<u8> = vec![0x80, 0x01];
    let res = data.as_slice().read_u32_leb_128().unwrap();
    assert_eq!(res, 128);

    let data: Vec<u8> = vec![0x40];
    let res = data.as_slice().read_u32_leb_128().unwrap();
    assert_eq!(res, 64);

    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F];
    let res = data.as_slice().read_u32_leb_128().unwrap();
    assert_eq!(res, 0xFFFFFFFF);

    let data: Vec<u8> = vec![0xF8, 0xFF, 0xFF, 0xFF, 0x0F];
    let res = data.as_slice().read_u32_leb_128().unwrap();
    assert_eq!(res, 0xFFFFFFF8);

    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
    let res = data.as_slice().read_u32_leb_128();
    assert_err!(res, LEB128Error::Overflow(_))?;

    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    let res = data.as_slice().read_u32_leb_128();
    assert_err!(res, LEB128Error::Unterminated(_))?;

    Ok(())
}

#[test]
fn test_leb128_u64() -> Result<()> {
    let data = vec![];
    let res = data.as_slice().read_u64_leb_128();
    assert_err!(
        res,
        LEB128Error::IOError(e),
        e.kind() == ErrorKind::UnexpectedEof
    )?;

    let data: Vec<u8> = vec![8];
    let res = data.as_slice().read_u64_leb_128().unwrap();
    assert_eq!(res, 8);

    let data: Vec<u8> = vec![0x80, 0x01];
    let res = data.as_slice().read_u64_leb_128().unwrap();
    assert_eq!(res, 128);

    let data: Vec<u8> = vec![0x40];
    let res = data.as_slice().read_u64_leb_128().unwrap();
    assert_eq!(res, 64);

    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
    let res = data.as_slice().read_u64_leb_128().unwrap();
    assert_eq!(res, 0xFFFFFFFFFFFFFFFF);

    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
    let res = data.as_slice().read_u64_leb_128().unwrap();
    assert_eq!(res, 0x7FFFFFFFFFFFFFFF);

    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
    let res = data.as_slice().read_u64_leb_128();
    assert_err!(res, LEB128Error::Overflow(_))?;

    let data: Vec<u8> = vec![
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ];
    let res = data.as_slice().read_u64_leb_128();
    assert_err!(res, LEB128Error::Unterminated(_))?;

    Ok(())
}

#[test]
fn test_leb128_i32() {
    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
    let res = data.as_slice().read_i32_leb_128().unwrap();
    assert_eq!(res, -1);

    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07];
    let res = data.as_slice().read_i32_leb_128().unwrap();
    assert_eq!(res, 0x7FFFFFFF);

    let data: Vec<u8> = vec![0x80, 0x7f];
    let res = data.as_slice().read_i32_leb_128().unwrap();
    assert_eq!(res, -128);

    let data: Vec<u8> = vec![0x80, 0x80, 0x80, 0x80, 0x78];
    let res = data.as_slice().read_i32_leb_128().unwrap();
    assert_eq!(res, -0x80000000);
}

#[test]
fn test_leb128_i64() {
    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
    let res = data.as_slice().read_i64_leb_128().unwrap();
    assert_eq!(res, 0x7FFFFFFFFFFFFFFF);

    let data: Vec<u8> = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x7F];
    let res = data.as_slice().read_i64_leb_128().unwrap();
    assert_eq!(res, -0x8000000000000000);

    let data: Vec<u8> = vec![0x80, 0x7f];
    let res = data.as_slice().read_i64_leb_128().unwrap();
    assert_eq!(res, -128);

    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
    let res = data.as_slice().read_i64_leb_128().unwrap();
    assert_eq!(res, -1);

    let data: Vec<u8> = vec![0x80, 0x7F];
    let res = data.as_slice().read_i64_leb_128().unwrap();
    assert_eq!(res, -128);
}
