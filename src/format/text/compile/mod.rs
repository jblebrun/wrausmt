use super::syntax::{self, Resolved};
use crate::{
    module,
    types::{FunctionType, ValueType},
};

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

impl From<syntax::ExportDesc<Resolved>> for module::ExportDesc {
    fn from(ast: syntax::ExportDesc<Resolved>) -> module::ExportDesc {
        match ast {
            syntax::ExportDesc::Func(idx) => module::ExportDesc::Func(idx.value()),
            syntax::ExportDesc::Table(idx) => module::ExportDesc::Table(idx.value()),
            syntax::ExportDesc::Mem(idx) => module::ExportDesc::Memory(idx.value()),
            syntax::ExportDesc::Global(idx) => module::ExportDesc::Func(idx.value()),
        }
    }
}

impl From<syntax::ExportField<Resolved>> for module::Export {
    fn from(ast: syntax::ExportField<Resolved>) -> module::Export {
        module::Export {
            name: ast.name,
            desc: ast.exportdesc.into(),
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

fn compile_function_body(func: &syntax::FuncField<Resolved>) -> Box<[u8]> {
    let mut out: Vec<u8> = Vec::new();

    out.emit_expr(&func.body);

    // ???
    // profit!
    out.into_boxed_slice()
}

fn compile_function(func: syntax::FuncField<Resolved>, types: &[FunctionType]) -> module::Function {
    // Get the typeidx from the finalized list of types
    let functype = match &func.typeuse.typeidx {
        Some(idx) => idx.value(),
        None => {
            let inline_def = &func.typeuse.get_inline_def().unwrap().into();
            types
                .iter()
                .position(|t| t == inline_def)
                .map(|p| p as u32)
                .unwrap_or_else(|| {
                    panic!("inline type missing from type list. this is a compiler error.")
                })
        }
    };

    // Convert the locals into a normal box
    let locals: Box<[ValueType]> = func.locals.iter().map(|l| l.valtype).collect();

    let body = compile_function_body(&func);

    // Compile the method body!!
    module::Function {
        functype,
        locals,
        body,
    }
}

pub fn compile(ast: syntax::Module<Resolved>) -> module::Module {
    let types: Box<[FunctionType]> = ast
        .types
        .into_iter()
        .map(|t| t.functiontype.into())
        .collect();

    let exports = ast.exports.into_iter().map(|e| e.into()).collect();

    // Needed:
    // * completed types list,
    // * completed index table,
    let funcs = ast
        .funcs
        .into_iter()
        .map(|f| compile_function(f, &types))
        .collect();

    module::Module {
        types,
        funcs,
        exports,
        ..module::Module::default()
    }
}
