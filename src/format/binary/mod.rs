use std::io::Bytes;
use std::io::Read;
use std::fs::File;
use std::io;
use super::super::module::Module;
use super::super::types::*;
use super::super::types::ValueType::*;
use super::super::types::NumType::*;
use super::super::types::RefType::*;

trait WasmParser: Read {
    fn next_leb_128(&mut self) -> io::Result<u32>;
    fn parse_result_type(&mut self) -> io::Result<Box<ResultType>>;
    fn expect_byte(&mut self, expect: u8) -> io::Result<u8>;
    fn parse_section_1(&mut self, len: u32) -> io::Result<Box<[FunctionType]>>;
    fn skip_section(&mut self, len: u32) -> io::Result<()> {
        let mut section: Vec<u8> = vec![0; len as usize];
        self.read_exact(&mut section)?;
        Ok(())
    }

    fn parse_section_0(&mut self, len: u32) -> io::Result<()> {
        let mut section: Vec<u8> = vec![0; len as usize];
        self.read_exact(&mut section)?;
        println!("CUSTOM: {:?}", section);
        Ok(())
    }


}

impl <I> WasmParser for I where I:Read {
    fn next_leb_128(&mut self) -> io::Result<u32> {
        let mut result: u32 = 0;
        let mut pos = 0;
        let mut completed = false;

        for next in self.bytes() {
            let x = next?;
            result |= ((x & 0x7f) as u32) << pos;
            if x & 0x80 == 0x00 { 
                completed = true;
                break;
            }
            pos += 7;
            if pos > 31 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "u32 LEB128 data is too long"));
            }
        }

        if !completed {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Reached end of input before parsing LEB128"));
        }
        
        return Ok(result);
    }
    
    fn parse_result_type(&mut self) -> io::Result<Box<ResultType>> {
        let item_count = self.next_leb_128()?;
        let mut vs: Vec<u8> = vec![0; item_count as usize];

        self.read_exact(&mut vs)?;

        vs.into_iter()
            .map(parse_value_type)
            .collect()
    }

    fn expect_byte(&mut self, expect: u8) -> io::Result<u8> {
        let actual = self.bytes().next().unwrap()?;
        match actual {
            expect => Ok(actual),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Expected {} but got {}", expect, actual)))
        }
    }

    fn parse_section_1(&mut self, len: u32) -> io::Result<Box<[FunctionType]>> { 
        let result: Vec<FunctionType>;
        let items = self.next_leb_128()?;

        (0..items).map(|_| {
            self.expect_byte(0x60)?;
            Ok(FunctionType {
                params: self.parse_result_type()?,
                result: self.parse_result_type()?
            })
        }).collect()
    }
}


pub fn parse_value_type(byte: u8) -> io::Result<ValueType> {
    match byte {
        0x7F => Ok(Num(I32)),
        0x7E => Ok(Num(I64)),
        0x7C => Ok(Num(F32)),
        0x7C => Ok(Num(F64)),
        0x70 => Ok(Ref(Func)),
        0x6F => Ok(Ref(Extern)),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("{} does not encode a ValueType", byte)))
    }
}


pub fn parse<R>(reader: &mut R) -> io::Result<Module> where R : Read {
    let mut buf: [u8; 4] = [0; 4];
    let magic_result = reader.read_exact(&mut buf);
    assert_eq!(buf, [0x00, 0x61, 0x73, 0x6D]);
    let version_result = reader.read_exact(&mut buf);
    assert_eq!(buf, [0x01, 0x00, 0x00, 0x00]);

    let mut module = Module {
        types: Box::new([]),
        funcs: Box::new([]),
        exports: Box::new([]),
    };

    loop {
        let section = match reader.bytes().next() {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Err(e),
            None => break
        };

        let len = reader.next_leb_128()?;
        println!("SECTION {} ({:x}) -- LENGTH {}", section, section, len);
        match section {
            0 => reader.parse_section_0(len)?,
            1 => module.types = reader.parse_section_1(len)?,
            _ => reader.skip_section(len)?
        }
       
    }
    Ok(module)
}

#[test]
fn test_parse_file_1() {
    let mut f = File::open("a.out.wasm").unwrap();
    let module = parse(&mut f);
    println!("MODULE! {:?}", module);

}

#[test]
fn test_leb128() {
    let data: Vec<u8> = vec![8];
    let res = data.as_slice().next_leb_128().unwrap();
    assert_eq!(res, 8);
    
    let data: Vec<u8> = vec![0x80, 0x01];
    let res = data.as_slice().next_leb_128().unwrap();
    assert_eq!(res, 128);

}


