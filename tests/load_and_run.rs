use std::fs::File;
use wrausmt::format::binary::{ParseError, parse};
use wrausmt::runtime::Runtime;
#[test]
fn simplefunc() -> Result<(), ParseError> {
    let mut runtime = Runtime::new();


    let mut f = File::open("testdata/simplefunc.wasm").unwrap();
    let test_mod = parse(&mut f)?;
    let mod2 = test_mod.clone();
    println!("MODULE {:x?}",test_mod);


    let mod_inst = runtime.load(test_mod);
    let res1 = runtime.call(mod_inst, "test", &[100u32.into()]).unwrap();
    assert_eq!(res1, 142u32.into());
    
    let mod_inst2 = runtime.load(mod2);
    let res2 = runtime.call(mod_inst2, "test", &[4u32.into()]).unwrap();
    assert_eq!(res2, 46u32.into());
    Ok(())
}

#[test]
fn locals() -> Result<(), ParseError> {
    let mut runtime = Runtime::new();

    let mut f = File::open("testdata/locals.wasm").unwrap();
    let test_mod = parse(&mut f)?;
    println!("MODULE {:x?}",test_mod);

    let mod_inst = runtime.load(test_mod);
    let res = runtime.call(mod_inst, "test", &[100u32.into()]).unwrap();
    assert_eq!(res, 500u32.into());

    Ok(())
}
