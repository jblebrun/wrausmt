/// Code generation utilities used by wrausmt.
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::create_dir_all;
use std::io::Result;
use std::io::BufReader;
use std::io::BufRead;
use std::collections::HashMap;
use std::io::Write;


/// The data for one instruction read from the instructions list file.
#[derive(Default, Debug)]
struct Instruction {
    /// The name of the instruction converted to a type-friendly name.
    typename: String,

    /// The name of the instruction in the ops file.
    name: String,

    /// The opcdoe of the instruction, as a hex number starting with 0x.
    opcode: String,

    /// The parseargs descriptor.
    parse_args: String,

    /// The body of the execution function.
    body: String
}

impl Instruction {
    /// Create a new [Instruction] from fields in the file.
    /// They should be ordered: |opcode, name, parseargs|.
    fn new(fields: Vec<&str>) -> Instruction {
        Instruction {
            typename: typename(fields[1]),
            name: fields[1].to_string(),
            opcode: fields[0].to_string(),
            parse_args: parse_args(fields[2]),
            body: String::new()
        }
    }
}

/// Read the master_ops_list.csv and emit functions, constant tables, function tables, and parse
/// args tables.
pub fn parse() -> Result<()> {
    let f = File::open("codegen/master_ops_list.csv")?;
    let buf_reader = BufReader::new(f);

    // The result containig a map of all instructions parsed.
    let mut insts = HashMap::new();

    // The current instruction being parsed.
    let mut curinst = Instruction::default();

    for rline in buf_reader.lines() {
        let wline = rline?;
        let line = wline.trim();
        
        // Skip empty lines.
        if line.is_empty() {
            continue;
        }

        // If a line starts with |, we're collecting the code for executing the instruction
        // most recently described above in the file. So just add this text to the 
        // body for the current instruction.
        if let Some(stripped_code_line) = line.strip_prefix('|') {
            let formatted_code_line = format!("   {}\n", stripped_code_line);
            curinst.body += &formatted_code_line;
            continue;
        }

        // Get the fields for an instruction descriptor, expecting 3: opcode, name, parse.
        let fields = line.split(',')
            .map(|l| l.trim())
            .collect::<Vec<&str>>();

        if fields.len() != 3 {
            println!("Unhandled line {}", line);
            continue;
        }

        // Update the current instruction, saving the old one so we can finalize it below.
        let oldinst = curinst;
        curinst = Instruction::new(fields);

        // If we were actually collecting an instruction (this wasn't the first one)
        // add it to the instructions lists.
        if !oldinst.typename.is_empty() {
            let opcode = hex(&oldinst.opcode);
            println!("OPCDOE FOR {} IS {}", oldinst.opcode, opcode);
            insts.insert(opcode, oldinst);
        }
    }

    // Emit the module wrapping the various generated items.
    emit_module()?;

    // Emit the file containing the code and descriptor structs.
    emit_code_file(&insts)?;

    // Emit the file containing the lookup array.
    emit_exec_table(&insts)?;

    // Emit the file containing the lookup array for instruction data, by opcode.
    emit_instruction_data_table(&insts)?;

    Ok(())
}   

pub static CODE_HEADER: &str = &"
use crate::runtime::exec::ExecutionContext;
use crate::runtime::exec::ExecutionContextActions;
use crate::error::Result;
";

/// Emit the file containing the instruction structures, including their execution functions.
fn emit_code_file(insts: &HashMap<u32, Instruction>) -> Result<()> {
    let mut f = OpenOptions::new()
        .write(true).create(true).truncate(true)
        .open("src/instructions/generated/instructions.rs")?;
    println!("WRITING {} TO {:?}", insts.len(), f);
    f.write_all(GEN_HEADER.as_bytes())?;
    f.write_all(CODE_HEADER.as_bytes())?;
    
    for (_, inst) in insts.iter() {
        let code = code_item(inst);
        println!("WRITING:\n{}", code);
        f.write_all(code.as_bytes())?;
    }

    Ok(())
}

fn emit_module() -> Result<()> {
    create_dir_all("src/instructions/generated")?;
    let mut f = OpenOptions::new()
        .write(true).create(true).truncate(true)
        .open("src/instructions/generated/mod.rs")?;
    f.write_all(GEN_HEADER.as_bytes())?;
    f.write_all("pub mod exec_table;\n".as_bytes())?;
    f.write_all("pub mod data_table;\n".as_bytes())?;
    f.write_all("pub mod instructions;\n".as_bytes())
}

pub static GEN_HEADER: &str = &"
/// This file was generated automatically by the codegen crate.
/// Do not edit it manually.
///
/// See build.rs for wrausmt or the included codegen crate for more details.
";

pub static EXEC_TABLE_HEADER: &str = &"
use crate::instructions::ExecFn;
use crate::instructions::unimpl;
use crate::instructions::bad;
use super::instructions;

pub static EXEC_TABLE: &[ExecFn] = &[
";

/// Emit the file containing the lookup table array. It generates an array with 256 entries,
/// and each entry in the array corresponds to one opcode.
fn emit_exec_table(insts: &HashMap<u32, Instruction>) -> Result<()> {
    let mut f = OpenOptions::new()
        .write(true).create(true).truncate(true)
        .open("src/instructions/generated/exec_table.rs")?;

    f.write_all(EXEC_TABLE_HEADER.as_bytes())?;

    for i in 0u32..256 {
        f.write_all(exec_table_item(insts.get(&i)).as_bytes())?;
    }
    
    f.write_all("];\n".as_bytes())?;

    Ok(())
}

pub static DATA_TABLE_HEADER: &str = &"
use crate::instructions::InstructionData;
use crate::instructions::ParseArgs;
use crate::instructions::BAD_INSTRUCTION;

pub static INSTRUCTION_DATA: &[&InstructionData] = &[
";

fn emit_instruction_data_table(insts: &HashMap<u32, Instruction>) -> Result<()> {
    let mut f = OpenOptions::new()
        .write(true).create(true).truncate(true)
        .open("src/instructions/generated/data_table.rs")?;
    
    f.write_all(DATA_TABLE_HEADER.as_bytes())?;

    for i in 0u32..256 {
        f.write_all(data_table_item(insts.get(&i)).as_bytes())?;
    }
    
    f.write_all("];\n".as_bytes())?;

    Ok(())
}

fn data_table_item(inst: Option<&Instruction>) -> String {
    match inst {
        Some(i) => format!("  &InstructionData {{ opcode: {}, name: \"{}\", parse_args: {} }},\n", 
            i.opcode,
            i.name,
            i.parse_args
        ),
        _ => "  &BAD_INSTRUCTION,\n".into()
    }
}

/// Emit one time in the lookup table.
fn exec_table_item(inst: Option<&Instruction>) -> String {
    match inst {
        None => "  bad,\n".into(),
        Some(i) if i.body.is_empty() => "  unimpl,\n".into(),
        Some(i) => format!("  instructions::{}_exec,\n", i.typename)
    }
}

fn code_item(inst: &Instruction) -> String {
    format!("
pub fn {typename}_exec(_ec: &mut ExecutionContext) -> Result<()> {{
  {body}    Ok(())
}}
",
    typename = inst.typename,
    body  = inst.body,
    )
}

/// Convert the function name into a type-friendly name.
/// Any punctuation is removed, and the initial character, along with any character following a
/// punctuation symbol is converted to uppercase.
pub fn typename(s: &str) -> String {
    let mut result = String::new();

    for ch in s.as_bytes().iter().map(|c| *c as char) {
        match ch {
            '.' | '_' => result.push('_'),
            _ => result.push(ch)
        }
    }

    result
}

fn parse_args(field: &str) -> String {
    match field {
        "()" => "ParseArgs::None".into(),
        "(u32)" => "ParseArgs::U32".into(),
        "(u32; u32)" => "ParseArgs::U32U32".into(),
        "(vu32)" => "ParseArgs::Vu32".into(),
        "(vu32; u32)" => "ParseArgs::Vu32U32".into(),
        "(d8)" => "ParseArgs::D8".into(),
        "(u64)" => "ParseArgs::U64".into(),
        "(f32)" => "ParseArgs::F32".into(),
        "(f64)" => "ParseArgs::F64".into(),
        "(d8; d8)" => "ParseArgs::D8D8".into(),
        "(u32; d8)" => "ParseArgs::U32D8".into(),
        _ => panic!("unknown parseargs {}", field)
    }
}
/// Quick-and-dirty hex parser. Doesn't do much validation.
pub fn hex(s: &str) -> u32 {
    let mut result = 0u32;
    let mut exp = 1u32;
    for digit in s.as_bytes().iter().rev() {
        match digit {
            b'0'..=b'9' => result += (digit - b'0') as u32 * exp,
            b'a'..=b'f' => result += (10 + digit - b'a') as u32 * exp,
            b'A'..=b'F' => result += (10 + digit - b'A') as u32 * exp,
            b'x' => break,
            _ => panic!("unhandling hex digit {}", digit)
        }
        exp *= 16;
    }
    result
}
