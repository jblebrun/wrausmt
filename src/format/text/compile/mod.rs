use super::syntax::{self,  Resolved};
use crate::{runtime::instance::{ExportInstance, ExternalVal}, types::{FunctionType, ValueType}};

const END_OPCODE: u8 = 0xb;

impl From<syntax::FunctionType> for FunctionType {
    fn from(ast: syntax::FunctionType) -> FunctionType {
        FunctionType {
            params: ast.params.iter().map(|p| p.valuetype).collect(),
            result: ast.results.iter().map(|r| r.valuetype).collect(),
        }
    }
}

impl From<syntax::Local> for ValueType {
    fn from(ast: syntax::Local) -> ValueType {
        ast.valtype
    }
}

impl From<syntax::ExportDesc<Resolved>> for ExternalVal {
    fn from(ast: syntax::ExportDesc<Resolved>) -> ExternalVal { 
        match ast {
            syntax::ExportDesc::Func(idx) => ExternalVal::Func(idx.value()),
            syntax::ExportDesc::Table(idx) => ExternalVal::Table(idx.value()),
            syntax::ExportDesc::Mem(idx) => ExternalVal::Memory(idx.value()),
            syntax::ExportDesc::Global(idx) => ExternalVal::Global(idx.value()),
        }
    }
}

impl From<syntax::ExportField<Resolved>> for ExportInstance {
    fn from(ast: syntax::ExportField<Resolved>) -> ExportInstance { 
        ExportInstance { 
            name: ast.name,
            addr: ast.exportdesc.into(),
        }
    }
}

trait Emitter {
    fn emit32(&mut self, v: u32);
    fn emit64(&mut self, v: u64);
    fn splice32(&mut self, idx: usize, v: u32);
    fn len(&self) -> usize;
    fn push(&mut self, b: u8);

    fn emit_block(
        &mut self,
        typeuse: &syntax::TypeUse<Resolved>, 
        expr: &syntax::Expr<Resolved>, 
        cnt: &syntax::Continuation
    ) {
        let self_arity = typeuse.functiontype.results.len();
        // For now: ignoring param types
        self.emit32(self_arity as u32);

        // If the continuation is at the start, we write self the current length
        // of the compiled selfput so far, adding 4 to account for the 4-byte value
        // that we are abself to write.
        if matches!(cnt, syntax::Continuation::Start) {
            // Account for the value we are abself to write.
            self.emit32(self.len() as u32 + 4);
        }

        // Remember the spot to write continuation, and reserve a spot.
        let continuation_location = self.len();
        self.emit32(0x00);

        self.emit_expr(expr);

        // If the continuation is at the end of the block, we go back to the space
        // we reserved (just above), and write self the position of the last item
        // in the current compiled selfput, which corresponds to the end of the expression.
        if matches!(cnt, syntax::Continuation::End) {
            self.splice32(continuation_location, self.len() as u32 - 1);
        }
    }

    fn emit_br_table(&mut self, indices: &[syntax::Index<Resolved, syntax::LabelIndex>]) {
        self.emit32(indices.len() as u32);
        for i in indices {
            self.emit32(i.value());
        }
    }

    fn emit_expr(&mut self, expr: &syntax::Expr<Resolved>) {
        for instr in &expr.instr {
            // Emit opcode
            self.push(instr.opcode);

            // Emit operands

            match &instr.operands {
                syntax::Operands::None => (),
                syntax::Operands::Block(_, typeuse, e, cnt) => self.emit_block(typeuse, e, cnt),
                syntax::Operands::BrTable(indices) => self.emit_br_table(indices),
                syntax::Operands::I32(n) => self.emit32(*n),
                syntax::Operands::I64(n) => self.emit64(*n),
                syntax::Operands::F32(n) => self.emit32(*n as u32),
                syntax::Operands::F64(n) => self.emit64(*n as u64),
                syntax::Operands::FuncIndex(idx) => self.emit32(idx.value()),
                syntax::Operands::TableIndex(idx) => self.emit32(idx.value()),
                syntax::Operands::GlobalIndex(idx) => self.emit32(idx.value()),
                syntax::Operands::ElemIndex(idx) => self.emit32(idx.value()),
                syntax::Operands::DataIndex(idx) => self.emit32(idx.value()),
                syntax::Operands::LocalIndex(idx) => self.emit32(idx.value()),
                syntax::Operands::LabelIndex(idx) => self.emit32(idx.value()),
                _ => panic!("Not yet implemented in compiler: {:?}", instr),
            }
        }
        self.push(END_OPCODE);
    }
}

impl Emitter for Vec<u8> {
    fn emit32(&mut self, v: u32) {
        let bytes = &v.to_le_bytes()[..];
        self.extend(bytes);
    }

    fn splice32(&mut self, idx: usize, v: u32) {
        let bytes = &v.to_le_bytes();
        self.splice(idx..idx + 4, bytes.iter().cloned());
    }

    fn emit64(&mut self, v: u64) {
        let bytes = &v.to_le_bytes()[..];
        self.extend(bytes);
    }

    fn push(&mut self, b: u8) { self.push(b) }
    fn len(&self) -> usize { self.len() }
}

pub fn compile_function_body(func: &syntax::FuncField<Resolved>) -> Box<[u8]> {
    let mut out: Vec<u8> = Vec::new();

    out.emit_expr(&func.body);

    // ???
    // profit!
    out.into_boxed_slice()
}
