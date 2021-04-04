mod error;
use std::io::{Read,Write};
use std::io;
use super::super::instructions::*;
use super::super::module::*;
use super::super::types::*;
use super::super::types::ValueType::*;
use super::super::types::NumType::*;
use super::super::types::RefType::*;
use error::*;


trait WasmParser: Read {
    fn skip_section(&mut self) -> Result<()> {
        let mut section: Vec<u8> = vec![];
        self.read_to_end(&mut section).wrap("reading skipped content")?;
        Ok(())
    }

    fn parse_custom_section(&mut self) -> Result<()> {
        let mut section: Vec<u8> = vec![];
        self.read_to_end(&mut section).wrap("reading custom content")?;
        println!("CUSTOM: {:?}", section);
        Ok(())
    }
   
    fn next_leb_128(&mut self) -> Result<u32> {
        let mut result: u32 = 0;
        let mut pos = 0;
        let mut completed = false;

        for next in self.bytes() {
            let x = next.wrap("reading leb 128 byte")?;
            result |= ((x & 0x7f) as u32) << pos;
            if x & 0x80 == 0x00 { 
                completed = true;
                break;
            }
            pos += 7;
            if pos > 31 {
                return Err(ParseError::new(format!("u32 LEB128 data is too long")));
            }
        }

        if !completed {
            return Err(ParseError::new(format!("Reached end of input before parsing LEB128")));
        }
        
        return Ok(result);
    }

    fn parse_name(&mut self) -> Result<String> {
        let length = self.next_leb_128().wrap("parsing length")?;
        
        let mut bs: Vec<u8> = vec![0; length as usize];

        self.read_exact(&mut bs).wrap("reading name data")?;

        String::from_utf8(bs).wrap("parsing name data")
    }
    
    fn parse_result_type(&mut self) -> Result<Box<ResultType>> {
        let item_count = self.next_leb_128().wrap("parsing count")?;

        (0..item_count).map(|_| {
            self.parse_value_type()
        }).collect()
    }

    fn expect_byte(&mut self, expect: u8) -> Result<u8> {
        let actual = self.parse_next_byte().wrap("parsing byte")?;
        if actual != expect {
            Err(ParseError::new(format!("Expected {} but got {}", expect, actual)))
        } else {
            Ok(actual)
        }
    }

    fn parse_next_byte(&mut self) -> Result<u8> {
        match self.bytes().next() {
            Some(Ok(b)) => Ok(b),
            Some(Err(e)) => Err(e),
            _ => Err(io::Error::new(io::ErrorKind::UnexpectedEof, ""))
        }.wrap("parsing next byte")
    }

    fn parse_types_section(&mut self) -> Result<Box<[FunctionType]>> { 
        let items = self.next_leb_128().wrap("parsing item count")?;

        (0..items).map(|_| {
            self.expect_byte(0x60).wrap("checking type byte")?;
            Ok(FunctionType {
                params: self.parse_result_type().wrap("parsing params")?,
                result: self.parse_result_type().wrap("parsing result")?
            })
        }).collect()
    }
    
    fn parse_imports_section(&mut self) -> Result<Box<[Import]>> {
        let items = self.next_leb_128().wrap("parsing count")?;

        (0..items).map(|_| {
            Ok(Import {
                module_name: self.parse_name().wrap("parsing module name")?,
                name: self.parse_name().wrap("parsing name")?, 
                desc: {
                    let kind = self.parse_next_byte().wrap("parsing kind")?;
                    match kind {
                        0 => ImportDesc::Func(self.next_leb_128().wrap("parsing func")?),
                        1 => ImportDesc::Table(self.parse_table_type().wrap("parsing table")?),
                        2 => ImportDesc::Memory(self.parse_memory_type().wrap("parsing memory")?),
                        3 => ImportDesc::Global(self.parse_global_type().wrap("parsing global")?),
                        _ => return Err(ParseError::new(format!("unknown import desc {}", kind)))
                    }
                }
            })

        }).collect()
    }

    fn parse_value_type(&mut self) -> Result<ValueType> {
        convert_value_type(self.parse_next_byte().wrap("fetching value type")?)
    }
    
    fn parse_ref_type(&mut self) -> Result<RefType> {
        convert_ref_type(self.parse_next_byte().wrap("fetching ref type")?)
    }

    fn parse_bool(&mut self) -> Result<bool> {
       let bool_byte = self.parse_next_byte().wrap("fetching bool")?;
       match bool_byte {
           0 => Ok(false),
           1 => Ok(true),
           _ => Err(ParseError::new(format!("invalid bool value {}", bool_byte)))
       }
    }

    fn parse_limits(&mut self) -> Result<Limits> {
        let has_upper = self.parse_bool().wrap("parsing has upper")?;
        Ok(Limits {
            lower: self.next_leb_128().wrap("parsing lower")?,
            upper: if has_upper { Some(self.next_leb_128().wrap("parsing upper")?) } else { None }
        })
    }

    fn parse_table_type(&mut self) -> Result<TableType> {
        Ok(TableType {
            reftype: self.parse_ref_type().wrap("parsing reftype")?,
            limits: self.parse_limits().wrap("parsing limits")?
        })
    }

    fn parse_global_type(&mut self) -> Result<GlobalType> {
        Ok(GlobalType {
            valtype: self.parse_value_type().wrap("parsing value")?,
            mutable: self.parse_bool().wrap("parsing mutable")?,
        })
    }

    fn parse_memory_type(&mut self) -> Result<MemType> {
        Ok(MemType {
            limits: self.parse_limits().wrap("parsing limits")?
        })
    }

    fn parse_funcs_section(&mut self) -> Result<Box<[TypeIndex]>> {
        let items = self.next_leb_128().wrap("parsing item count")?;
        (0..items).map(|_| {
            self.next_leb_128().wrap("parsing func")
        }).collect()
    }

    fn parse_code_section(&mut self, types: &Box<[TypeIndex]>) -> Result<Box<[Function]>> {
       let items = self.next_leb_128().wrap("parsing item count")?;
       (0..items).map(|i| {
            let codesize = self.next_leb_128().wrap("parsing func")?;
            println!("\nFUNCTION {}", i);
            println!("CODE SIZE {}", codesize);
            let mut code_reader = self.take(codesize as u64);
            Ok(
                Function {
                    functype: types[i as usize],
                    locals: code_reader.parse_locals().wrap("parsing locals")?,
                    body: code_reader.take(codesize as u64).parse_code().wrap("parsing code")?
                }
            )
        }).collect()
    }

    fn parse_locals(&mut self) -> Result<Box<[ValueType]>> {
        let items = self.next_leb_128().wrap("parsing item count")?;
        let mut result: Vec<ValueType> = vec![];

        for _ in 0..items {
            let reps = self.next_leb_128().wrap("parsing type rep")?;
            let val = self.parse_value_type().wrap("parsing value type")?;
            for _ in 0..reps {
                result.push(val);
            }
        }
        Ok(result.into_boxed_slice())
    }

    fn next_idx(&mut self) -> Result<u32> {
        self.next_leb_128().wrap("parsing index argument")
    }

    fn parse_inst<W : Write>(&mut self, out: &mut W) -> Result<()> {
        let mut opcode_buf = [0u8; 1];
        self.read_exact(&mut opcode_buf).wrap("parsing opcode")?;
        let opcode = opcode_buf[0];
        
        // Assume success, write out the opcode. Validation occurs later.
        out.write(&opcode_buf).wrap("writing opcode")?;

        // Handle any additional behavior
        #[allow(non_upper_case_globals)]
        match opcode {
            Block => self.emit_next_leb_128(out)?,
            BrIf => self.emit_next_leb_128(out)?,
            Call => self.emit_next_leb_128(out)?,
            CallIndirect => {
                self.emit_next_leb_128(out)?;
                self.emit_next_leb_128(out)?;
            },
            _ => ()
        }
        Ok(())
    }

    fn emit_next_leb_128<W : Write>(&mut self, out: &mut W) -> Result<()> {
        out.write(
            &self.next_leb_128().wrap("reading leb 128")?.to_le_bytes()
        ).wrap("writing leb 128")?;
        Ok(())
    }

    fn parse_code(&mut self) -> Result<Box<[u8]>> {
        let result: Vec<u8> = vec![];
        Ok(result.into_boxed_slice())
    }
}

impl <I> WasmParser for I where I:Read {}

#[allow(dead_code)]
pub fn convert_number_type(byte: u8) -> Result<NumType> {
    match byte {
        0x7F => Ok(I32),
        0x7E => Ok(I64),
        0x7D => Ok(F32),
        0x7C => Ok(F64),
        _ => Err(ParseError::new(format!("{} does not encode a NumType", byte)))
    }
}

#[allow(dead_code)]
pub fn convert_ref_type(byte: u8) -> Result<RefType> {
    match byte {
        0x70 => Ok(Func),
        0x6F => Ok(Extern),
        _ => Err(ParseError::new(format!("{} does not encode a RefType", byte)))
    }
}

#[allow(dead_code)]
pub fn convert_value_type(byte: u8) -> Result<ValueType> {
    match byte {
        0x7F => Ok(Num(I32)),
        0x7E => Ok(Num(I64)),
        0x7D => Ok(Num(F32)),
        0x7C => Ok(Num(F64)),
        0x70 => Ok(Ref(Func)),
        0x6F => Ok(Ref(Extern)),
        _ => Err(ParseError::new(format!("{} does not encode a ValueType", byte)))
    }
}

struct CountRead<T> {
    inner: T,
    consumed: usize
}

impl <T> CountRead<T> {
    fn new(inner: T) -> CountRead<T> {
        CountRead {
            inner,
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

#[allow(dead_code)]
pub fn parse<R>(src: &mut R) -> Result<Module> where R : Read {
    let reader = &mut CountRead::new(src);

    let mut buf: [u8; 4] = [0; 4];
    reader.read_exact(&mut buf).wrap("reading magic")?;
    assert_eq!(buf, [0x00, 0x61, 0x73, 0x6D]);
    reader.read_exact(&mut buf).wrap("reading version")?;
    assert_eq!(buf, [0x01, 0x00, 0x00, 0x00]);

    let mut module = Module {
        types: Box::new([]),
        imports: Box::new([]),
        funcs: Box::new([]),
        exports: Box::new([]),
    };

    let mut functypes: Box<[TypeIndex]> = Box::new([]);

    fn parse_section<R : Read>(
        section: u8, 
        module: &mut Module, 
        reader: &mut R,
        functypes: &mut Box<[TypeIndex]>
    ) -> Result<()> {
        let len = reader.next_leb_128().wrap("parsing length")?;
        println!("SECTION {} ({:x}) -- LENGTH {}", section, section, len);
        let mut section_reader = reader.take(len as u64);
        match section {
            0 => section_reader.parse_custom_section().wrap("parsing custom")?,
            1 => module.types = section_reader.parse_types_section().wrap("parsing types")?,
            2 => module.imports = section_reader.parse_imports_section().wrap("parsing imports")?,
            3 => *functypes = section_reader.parse_funcs_section().wrap("parsing funcs")?,
            10 => module.funcs = section_reader.parse_code_section(functypes).wrap("parsing code")?,
            _ => section_reader.skip_section().wrap("skipping section")?
        }
        let mut remaining: Vec<u8> = vec![];
        section_reader.read_to_end(&mut remaining).wrap("check remaining")?;
        if remaining.len() > 0 {
            Err(ParseError::new(format!("Section {} was not fully consumed. {:x?} remaining.", section, remaining)))
        } else {
            Ok(())
        }
    }

    loop {
        let section = match reader.bytes().next() {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Err(e).wrap("parsing section"),
            None => break
        };

        match parse_section(section, &mut module, reader, &mut functypes) {
            Err(e) => {
                let consumed = reader.consumed();
                println!("\n\nError parsing at {} (0x{:x}), caused by:\n{}\n\n", consumed, consumed, e);
                break;
            }
            _ => ()
        }
    }
    println!("FUNCTYPES {:?}", functypes);
    Ok(module)
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use super::*;

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

        let data: Vec<u8> = vec![0x40];
        let res = data.as_slice().next_leb_128().unwrap();
        assert_eq!(res, 64);
    }
}
