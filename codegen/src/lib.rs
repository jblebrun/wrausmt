//! Code generation utilities used by wrausmt.

mod code;
mod data_table;
mod exec_table;
mod fields;
use {
    code::EmitCode,
    data_table::EmitDataTable,
    exec_table::EmitExecTable,
    std::{
        fs,
        io::{self, BufRead, Write},
    },
};

/// The data for one instruction read from the instructions list file.
#[derive(Debug)]
pub struct Instruction {
    /// The name of the instruction converted to a type-friendly name.
    typename: String,

    /// The name of the instruction in the ops file.
    name: String,

    /// The opcode of the instruction, as a hex number starting with 0x.
    opcode: u8,

    /// The operands descriptor.
    operands: String,

    /// The body of the execution function.
    body: String,
}

impl Default for Instruction {
    fn default() -> Self {
        Self {
            typename: "".into(),
            name:     "ILLEGAL".into(),
            operands: "None".into(),
            opcode:   0,
            body:     "".into(),
        }
    }
}

impl Instruction {
    /// Create a new [Instruction] from fields in the file.
    /// They should be ordered: |opcode, name, operands|.
    fn new(fields: Vec<&str>) -> Instruction {
        Instruction {
            typename: fields::typename(fields[1]),
            name:     fields[1].to_string(),
            opcode:   fields::hex(fields[0]) as u8,
            operands: fields::operands(fields[2]),
            body:     String::new(),
        }
    }
}

enum Variant {
    Normal,
    Extended,
    Simd,
}

impl Variant {
    fn prefix(&self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Extended => "EXTENDED_",
            Self::Simd => "SIMD_",
        }
    }

    fn opcode_variant(&self) -> &'static str {
        match self {
            Self::Normal => "Opcode::Normal",
            Self::Extended => "Opcode::Extended",
            Self::Simd => "Opcode::Simd",
        }
    }
}
struct InstructionsForVariant {
    variant:      Variant,
    instructions: [Option<Instruction>; 256],
}

fn read_instruction_list(file: &str, variant: Variant) -> io::Result<InstructionsForVariant> {
    let f = fs::File::open(file)?;
    let buf_reader = io::BufReader::new(f);

    // The result containig a map of all instructions parsed.
    let mut instructions = InstructionsForVariant {
        variant,
        instructions: core::array::from_fn(|_| None),
    };

    // The current instruction being parsed.
    let mut curinst = Instruction::default();

    for rline in buf_reader.lines() {
        let wline = rline?;
        let line = wline.trim();

        // Skip empty lines.
        if line.is_empty() {
            continue;
        }

        // If a line starts with |, we're collecting the code for executing the
        // instruction most recently described above in the file. So just add
        // this text to the body for the current instruction.
        if let Some(stripped_code_line) = line.strip_prefix('|') {
            let formatted_code_line = if stripped_code_line.is_empty() {
                "".to_owned()
            } else {
                format!("   {}\n", stripped_code_line)
            };
            curinst.body += &formatted_code_line;
            continue;
        }

        // Get the fields for an instruction descriptor, expecting 3: opcode, name,
        // parse.
        let fields = line.split(',').map(|l| l.trim()).collect::<Vec<&str>>();

        if fields.len() != 3 {
            println!("Unhandled line {}", line);
            continue;
        }

        // Update the current instruction, saving the old one so we can finalize it
        // below.
        let oldinst = curinst;
        curinst = Instruction::new(fields);

        // If we were actually collecting an instruction (this wasn't the first one)
        // add it to the instructions lists.
        if !oldinst.typename.is_empty() {
            let opcode = oldinst.opcode;
            instructions.instructions[opcode as usize] = Some(oldinst);
        }
    }

    Ok(instructions)
}

/// Read master_ops_list.csv and emit functions, function tables, and data
/// tables.
pub fn parse() -> io::Result<()> {
    let inst_groups = &[
        read_instruction_list("codegen/master_ops_list.csv", Variant::Normal)?,
        read_instruction_list("codegen/master_extended_ops_list.csv", Variant::Extended)?,
        read_instruction_list("codegen/master_simd_ops_list.csv", Variant::Simd)?,
    ];

    fs::create_dir_all("src/instructions/generated")?;

    // Emit the module wrapping the various generated items.
    emit_module()?;

    // Emit the file containing the code and descriptor structs.
    let mut code_file = new_output_file("src/instructions/generated/instructions.rs")?;
    code_file.emit_code_file(inst_groups)?;

    // Emit the file containing the lookup array.
    let mut code_file = new_output_file("src/instructions/generated/exec_table.rs")?;
    code_file.emit_exec_table(inst_groups)?;

    // Emit the file containing the lookup array for instruction data, by opcode.
    let mut code_file = new_output_file("src/instructions/generated/data_table.rs")?;
    code_file.emit_instruction_data_table(inst_groups)?;

    Ok(())
}

fn new_output_file(name: &str) -> io::Result<fs::File> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(name)?;

    f.write_all(GEN_HEADER)?;
    Ok(f)
}

fn emit_module() -> io::Result<()> {
    let mut f = new_output_file("src/instructions/generated/mod.rs")?;
    f.write_all(GEN_HEADER)?;
    f.write_all(MODULE)
}

pub static MODULE: &[u8] = br#"pub mod data_table;
pub mod exec_table;
pub mod instructions;
"#;

pub static GEN_HEADER: &[u8] = br#"/// This file was generated automatically by the codegen crate.
/// Do not edit it manually.
///
/// See build.rs for wrausmt or the included codegen crate for more details.
"#;
