f = open("ops.csv", "r")

code_template = """
struct {typename} {{ }}

impl Instruction for {typename} {{
  fn data() -> InstructionData {{
    opcode: {opcode},
    name: "{name}",
    parse_args
  }}

  fn exec<E:ExecutionContextActions>(ec: &mut E) -> Result<()> {{
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



def emit_rust_code(fields):
    args = {
            "typename": type_name(fields[1]),
            "name": fields[1],
            "opcode": fields[0],
            "body": "//coming soon"
    }
    #print(code_template.format(**args))
    print("""
#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const {typename}:u8 = {opcode};
    """.format(**args))

for line in f:
    fields = line.split(",")
    if fields[0] == "OPCODE": continue

    if len(fields) < 3:
        continue
    fields = [f.strip() for f in fields]
    print("{:<10},{:<30},{}".format(*fields))
    #emit_rust_code(fields)





