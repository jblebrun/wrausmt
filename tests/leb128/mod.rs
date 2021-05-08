use wrausmt::assert_err_match;
use wrausmt::format::binary::leb128::ReadLeb128;

#[test]
fn test_leb128_u32() {
    let data = vec![];
    let res = data.as_slice().read_u32_leb_128();
    assert_err_match!(res, "did not reach terminal LEB128 byte");

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
    assert_err_match!(res, "value overflows");

    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    let res = data.as_slice().read_u32_leb_128();
    assert_err_match!(res, "did not reach terminal LEB128 byte");
}

#[test]
fn test_leb128_u64() {
    let data = vec![];
    let res = data.as_slice().read_u64_leb_128();
    assert_err_match!(res, "did not reach terminal LEB128 byte");

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
    assert_err_match!(res, "value overflows");

    let data: Vec<u8> = vec![
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ];
    let res = data.as_slice().read_u64_leb_128();
    assert_err_match!(res, "did not reach terminal LEB128 byte");
}

#[test]
fn test_leb128_i32() {
    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F];
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

    let data: Vec<u8> = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
    let res = data.as_slice().read_i64_leb_128().unwrap();
    assert_eq!(res, -0x8000000000000000);

    let data: Vec<u8> = vec![0x80, 0x7f];
    let res = data.as_slice().read_i64_leb_128().unwrap();
    assert_eq!(res, -128);

    let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
    let res = data.as_slice().read_i64_leb_128().unwrap();
    assert_eq!(res, -1);

    let data: Vec<u8> = vec![0x80, 0x7F];
    let res = data.as_slice().read_i64_leb_128().unwrap();
    assert_eq!(res, -128);
}
