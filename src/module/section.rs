use crate::{
    module::{index, Import, Memory, Start, Table},
    module::{Data, Elem, Export, Function, Global},
    types::FunctionType,
};

#[derive(Debug)]
pub enum Section {
    Eof,
    Skip,
    Custom(Box<[u8]>),
    Types(Box<[FunctionType]>),
    Imports(Box<[Import]>),
    Funcs(Box<[index::Type]>),
    Tables(Box<[Table]>),
    Mems(Box<[Memory]>),
    Globals(Box<[Global]>),
    Exports(Box<[Export]>),
    Start(Option<Start>),
    Elems(Box<[Elem]>),
    Code(Box<[Function]>),
    Data(Box<[Data]>),
    DataCount(u32),
}
