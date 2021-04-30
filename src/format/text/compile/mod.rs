use super::syntax::{self, Index, Resolved};
use crate::{module, types::{FunctionType, ValueType}};
use crate::error::{Result, ResultFrom};
use crate::error;
use std::io::Write;

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
            syntax::ExportDesc::Func(idx) => module::ExportDesc::Func(idx.value),
            syntax::ExportDesc::Table(idx) => module::ExportDesc::Table(idx.value),
            syntax::ExportDesc::Mem(idx) => module::ExportDesc::Memory(idx.value),
            syntax::ExportDesc::Global(idx) => module::ExportDesc::Func(idx.value)
        }
    }
}

impl From<syntax::ExportField<Resolved>> for module::Export {
    fn from(ast: syntax::ExportField<Resolved>) -> module::Export {
        module::Export {
            name: ast.name,
            desc: ast.exportdesc.into()
        }
    }
}

fn compile_function_body(func: &syntax::FuncField<Resolved>) -> Result<Box<[u8]>> {
    let mut body: Vec<u8> = Vec::new();

    for instr in &func.body.instr {
        // Emit opcode
        body.write(&[instr.opcode]).wrap("writing opcode")?;
        // Emit operands
        match instr.operands {
            syntax::Operands::None => (),
            syntax::Operands::I32(n) |
            syntax::Operands::FuncIndex(Index { value:n, .. }) |
            syntax::Operands::TableIndex(Index { value:n, ..}) |
            syntax::Operands::GlobalIndex(Index { value:n, ..}) | 
            syntax::Operands::ElemIndex(Index { value:n, ..}) |
            syntax::Operands::DataIndex(Index { value:n, ..}) |
            syntax::Operands::LocalIndex(Index { value:n, ..}) |
            syntax::Operands::LabelIndex(Index { value:n, ..}) => {
                let bytes = &n.to_le_bytes()[..];
                body.write(&bytes).wrap("writing index operand")?;
            }
            _ => ()
        }
    }
    // ???
    // profit!
    Ok(body.into_boxed_slice())
}

fn compile_function(
    func: syntax::FuncField<Resolved>,
    types: &[FunctionType],
) -> Result<module::Function> {

    // Get the typeidx from the finalized list of types
    let functype = match &func.typeuse.typeidx {
        Some(idx) => Ok(idx.value),
        None => {
            let inline_def = &func.typeuse.get_inline_def().unwrap().into();
            types.iter()
                .position(|t| t == inline_def)
                .map(|p| p as u32)
                .ok_or_else(|| error!("inline type missing from type list. this is a compiler error."))
        }
    }?;

    // Convert the locals into a normal box
    let locals: Box<[ValueType]> = func.locals.iter().map(|l| l.valtype).collect();

    let body = compile_function_body(&func)?;

    // Compile the method body!!
    Ok(module::Function {
        functype,
        locals,
        body
    })
}

pub fn compile(ast: syntax::Module<Resolved>) -> Result<module::Module> {
    let types: Box<[FunctionType]> = 
        ast.types.into_iter().map(|t| t.functiontype.into()).collect();

    let exports = ast.exports.into_iter().map(|e| e.into()).collect();

    // Needed: 
    // * completed types list,
    // * completed index table,
    let funcs = ast.funcs
        .into_iter()
        .map(|f| compile_function(f, &types))
        .collect::<Result<Box<[module::Function]>>>()?;

    Ok(module::Module {
        types,
        funcs, 
        exports,
        ..module::Module::default()
    })
}
