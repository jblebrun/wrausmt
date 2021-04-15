
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

  fn exec<E:ExecutionContextActions>(_ec: &mut E) -> Result<()> {{
{body}
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

def emit_rust_code(instr):
    print(code_template.format(**instr))

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
    print("use crate::runtime::exec::ExecutionContextActions;")
    print("use crate::instructions::Instruction;")
    print("use crate::instructions::InstructionData;")
    print("use crate::instructions::ParseArgs;")
    print("use crate::error::Result;")

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

        if curinst is not None:
            curinst["body"] += "   Ok(())"
            emit_rust_code(curinst)
        curinst = instr(fields)

        #print("{:<10},{:<30},{}".format(*fields))
        #emit_rust_code(fields)


if __name__ == "__main__":
    parse()





