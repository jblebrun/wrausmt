use wrausmt::format::text::parse::Parser;
use wrausmt::format::text::compile::compile;
use wrausmt::format::text::lex::Tokenizer;
use wrausmt::error::{Result, ResultFrom};
use wrausmt::runtime::Runtime;

#[test]
fn simplefunc() -> Result<()> {
    let f = std::fs::File::open("testdata/simplefunc.wat").wrap("opening file")?;

    let tokenizer = Tokenizer::new(f)?;
    let mut parser = Parser::new(tokenizer)?;
    let ast = parser.try_module()?.unwrap();
    
    println!("AST! {:?}", ast);
    let module = compile(ast)?;
    
    println!("MODULE! {:?}", module);

    let mut runtime = Runtime::new();
    let mod_inst = runtime.load(module)?;

    let res1 = runtime.call(&mod_inst, "test", &[100u32.into()])?;
    assert_eq!(res1, 142u32.into());

    Ok(())
}
