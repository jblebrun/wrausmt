use std::io::Bytes;
use std::io::Read;
use std::io::BufReader;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use super::super::module::*;
use super::super::types::*;
use super::super::types::ValueType::*;
use super::super::types::NumType::*;
use super::super::types::RefType::*;

trait WasmParser: Read {
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

    fn parse_name(&mut self) -> io::Result<String> {
        let length = self.next_leb_128()?;
        
        let mut bs: Vec<u8> = vec![0; length as usize];

        self.read_exact(&mut bs)?;

        match String::from_utf8(bs) {
            Ok(item) => Ok(item),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, "Utf8 error"))
        }
    }
    
    fn parse_result_type(&mut self) -> io::Result<Box<ResultType>> {
        let item_count = self.next_leb_128()?;

        (0..item_count).map(|_| {
            self.parse_value_type()
        }).collect()
    }

    fn expect_byte(&mut self, expect: u8) -> io::Result<u8> {
        let actual = self.parse_next_byte()?;
        match actual {
            expect => Ok(actual),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("Expected {} but got {}", expect, actual)))
        }
    }

    fn parse_next_byte(&mut self) -> io::Result<u8> {
        match self.bytes().next() {
            Some(Ok(b)) => Ok(b),
            Some(Err(e)) => Err(e),
            _ => Err(io::Error::new(io::ErrorKind::UnexpectedEof, ""))
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
    
    /// Section 2: Imports
    fn parse_section_2(&mut self) -> io::Result<Box<[Import]>> {
        let items = self.next_leb_128()?;

        (0..items).map(|_| {
            Ok(Import {
                module_name: self.parse_name()?,
                name: self.parse_name()?, 
                desc: {
                    let kind = self.parse_next_byte()?;
                    match kind {
                        0 => ImportDesc::Func(self.next_leb_128()?),
                        1 => ImportDesc::Table(self.parse_table_type()?),
                        2 => ImportDesc::Memory(self.parse_memory_type()?),
                        3 => ImportDesc::Global(self.parse_global_type()?),
                        _ => return invalid_data(format!("unknown import desc {}", kind))
                    }
                }
            })

        }).collect()
    }

    fn parse_value_type(&mut self) -> io::Result<ValueType> {
        convert_value_type(self.parse_next_byte()?)
    }
    
    fn parse_ref_type(&mut self) -> io::Result<RefType> {
        convert_ref_type(self.parse_next_byte()?)
    }

    fn parse_bool(&mut self) -> io::Result<bool> {
       let bool_byte = self.parse_next_byte()?;
       match bool_byte {
           0 => Ok(false),
           1 => Ok(true),
           _ => invalid_data(format!("invalid bool value {}", bool_byte))
       }
    }

    fn parse_limits(&mut self) -> io::Result<Limits> {
        let has_upper = self.parse_bool()?;
        Ok(Limits {
            lower: self.next_leb_128()?,
            upper: if has_upper { Some(self.next_leb_128()?) } else { None }
        })
    }

    fn parse_table_type(&mut self) -> io::Result<TableType> {
        Ok(TableType {
            reftype: self.parse_ref_type()?,
            limits: self.parse_limits()?
        })
    }

    fn parse_global_type(&mut self) -> io::Result<GlobalType> {
        Ok(GlobalType {
            valtype: self.parse_value_type()?,
            mutable: self.parse_bool()?
        })
    }

    fn parse_memory_type(&mut self) -> io::Result<MemType> {
        Ok(MemType {
            limits: self.parse_limits()?
        })
    }
}

impl <I> WasmParser for I where I:Read {}

fn invalid_data<T>(msg: String) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::InvalidData, msg))
}

pub fn convert_number_type(byte: u8) -> io::Result<NumType> {
    match byte {
        0x7F => Ok(I32),
        0x7E => Ok(I64),
        0x7C => Ok(F32),
        0x7C => Ok(F64),
        _ => invalid_data(format!("{} does not encode a NumType", byte))
    }
}

pub fn convert_ref_type(byte: u8) -> io::Result<RefType> {
    match byte {
        0x70 => Ok(Func),
        0x6F => Ok(Extern),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("{} does not encode a RefType", byte)))
    }
}

pub fn convert_value_type(byte: u8) -> io::Result<ValueType> {
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

struct CountRead<T> {
    inner: T,
    consumed: usize
}

impl <T> CountRead<T> {
    fn new(inner: T) -> CountRead<T> {
        CountRead {
            inner: inner,
            consumed: 0
        }
    }

    pub fn consumed(&self) -> usize { self.consumed }
}

impl <T : Read> Read for CountRead<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner.read(buf) {
            Ok(c) => { self.consumed += c; Ok(c) },
            Err(e) => Err(e)
        }
    }
}

pub fn parse<R>(src: &mut R) -> io::Result<Module> where R : Read {
    let reader = &mut CountRead::new(src);

    let mut buf: [u8; 4] = [0; 4];
    let magic_result = reader.read_exact(&mut buf);
    assert_eq!(buf, [0x00, 0x61, 0x73, 0x6D]);
    let version_result = reader.read_exact(&mut buf);
    assert_eq!(buf, [0x01, 0x00, 0x00, 0x00]);

    let mut module = Module {
        types: Box::new([]),
        imports: Box::new([]),
        funcs: Box::new([]),
        exports: Box::new([]),
    };

    fn parse_section<R : Read>(
        section: u8, 
        module: &mut Module, 
        reader: &mut R,
    ) -> io::Result<()> {
        let len = reader.next_leb_128()?;
        println!("SECTION {} ({:x}) -- LENGTH {}", section, section, len);
        match section {
            0 => reader.parse_section_0(len)?,
            1 => module.types = reader.parse_section_1(len)?,
            2 => module.imports = reader.parse_section_2()?,
            _ => reader.skip_section(len)?
        }
        Ok(())
    }

    loop {
        let section = match reader.bytes().next() {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Err(e),
            None => break
        };

        match parse_section(section, &mut module, reader) {
            Err(e) => {
                let consumed = reader.consumed();
                println!("Error parsing: {:?} at {} (0x{:x})", e, consumed, consumed);
                break;
            }
            _ => ()
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


