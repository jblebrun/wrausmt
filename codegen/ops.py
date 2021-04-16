
code_template = """
pub struct {typename} {{ }}

impl Instruction for {typename} {{
  fn data() -> InstructionData {{
    InstructionData {{
      opcode: {opcode},
      name: "{name}",
      parse_args: {parseargs},
    }}
  }}

  fn exec(_ec: &mut ExecutionContext) -> Result<()> {{
{body}
    Ok(())
  }}
}}
"""

def type_name(name):
    last = '.'
    result = []
    for c in name:
        if last in ('.','_'):
            result.append(c.upper())
        else: 
            if c not in ('.','_'):
                result.append(c)
        last = c
    return "".join(result)

def parse_args_name(field):
    f = field.strip()
    if f == "(u32)":
        return "ParseArgs::U32"
    if f == "(u32, u32)":
        return "ParseArgs::U32U32"
    return "ParseArgs::None"

def emit_rust_code(instr, f):
    f.write(code_template.format(**instr))

def emit_const_table(instr):
    print("""
#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const {typename}:u8 = {opcode};
    """.format(**args))

def instr(fields):
    return {
            "typename": type_name(fields[1]),
            "name": fields[1],
            "opcode": fields[0],
            "parseargs": parse_args_name(fields[2]),
            "body": ""
    }

def parse():
    f = open("codegen/master_ops_list.csv", "r")
    curinst = None

    opcode_to_instr = {}

    for line in f:
        if line[0] == "|":
            if curinst == None:
                raise ValueError("Code without instruction")
            curinst["body"] += "   {}".format(line[1:])
            continue

        fields = line.split(",")
        if len(fields) < 3:
            continue
        fields = [f.strip() for f in fields]

        if fields[0] == "OPCODE": continue

        opnum = int(fields[0], 16)
        curinst = instr(fields)
        
        opcode_to_instr[opnum] = curinst

    return opcode_to_instr

def emit_instructions_code_file(insts):
    f = open("src/instructions/gentypes.rs", "w")
    f.write("use crate::runtime::exec::ExecutionContext;\n")
    f.write("use crate::runtime::exec::ExecutionContextActions;\n")
    f.write("use crate::instructions::Instruction;\n")
    f.write("use crate::instructions::InstructionData;\n")
    f.write("use crate::instructions::ParseArgs;\n")
    f.write("use crate::error::Result;\n")
    for inst in insts:
        emit_rust_code(insts[inst], f)
    f.close()

def emit_exec_table(insts):
    f = open("src/instructions/exec_table.rs", "w")
    f.write("use crate::instructions::ExecFn;\n")
    f.write("use crate::instructions::Instruction;\n")
    f.write("use crate::instructions::unimpl;\n")
    f.write("use crate::instructions::bad;\n")
    f.write("use crate::instructions::gentypes;\n\n")
    f.write("pub static EXEC_TABLE: &[ExecFn] = &[\n")
    for i in range(0,256):
        if i in insts:
            if insts[i]["body"] == "":
                f.write("  unimpl,\n")
            else:
                f.write("  gentypes::{typename}::exec,\n".format(**insts[i]))
        else:
            f.write("  bad,\n")
    f.write("];\n")
    f.close()

if __name__ == "__main__":
    insts = parse()

    emit_instructions_code_file(insts)

    emit_exec_table(insts)




