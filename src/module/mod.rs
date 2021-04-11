//! Definitions for a WASM Module.
//!
//! [From Spec][Spec]:
//!  > WebAssembly programs are organized into modules, which are the unit of deployment, loading,
//!  > and compilation. A module collects definitions for types, functions, tables, memories, and
//!  > globals. In addition, it can declare imports and exports and provide initialization in the
//!  > form of data and element segments, or a start function.
//!
//!
//! The organizaton of this structure matches definition of a module as described in the [WASM
//! Spec][Spec]. The documentation for most
//! items is quoted directly from this spec.
//!
//! The types described in the spec are mapped pretty closely to Rust struct, enum, and type
//! declarations, as appropriate. The type `vec(T)` maps to a [Box<T>]. To avoid confusion with the
//! Rust standard library's vector type, a Vec type is not defined here.
//!
//! [Spec]: https://webassembly.github.io/spec/core/syntax/modules.html
use super::types::{FunctionType, GlobalType, MemType, RefType, TableType, ValueType};
use crate::instructions::Expr;

/// The struct containing all of the information relevant to one WASM module. [Spec][Spec]
///
/// The [Module] is the core representation of a WASM module that can be loaded in to the
/// runtime. This also serves as the output representation for the two parsers. Both the binary and
/// the text format parsers will generate this module struct as the canonical output of their
/// parsing of an input format.
///
/// Each of the vectors – and thus the entire module – may be empty.
///
/// See the documentation for the contained type of each field for more information.
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/modules.html
#[derive(Default, Debug, Clone)]
pub struct Module {
    pub types: Box<[Type]>,
    pub imports: Box<[Import]>,
    pub funcs: Box<[Function]>,
    pub tables: Box<[Table]>,
    pub mems: Box<[Memory]>,
    pub globals: Box<[Global]>,
    pub exports: Box<[Export]>,
    pub start: Option<Start>,
    pub elems: Box<[Elem]>,
    pub datas: Box<[Data]>,
}

/// A module namespace for the various index types used in module type definitions. [Spec][Spec]
///
/// Definitions are referenced with zero-based indices. Each class of definition has its own index
/// space, as distinguished by the following classes.
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/modules.html#indices
pub mod index {
    /// The index space for types includes respective imports declared in the same module.  The
    /// indices of these imports precede the indices of other definitions in the same index space.
    pub type Type = u32;

    /// The index space for functions includes respective imports declared in the same module.  The
    /// indices of these imports precede the indices of other definitions in the same index space.
    pub type Func = u32;

    /// The index space for tables includes respective imports declared in the same module. The
    /// indices of these imports precede the indices of other definitions in the same index space.
    pub type Table = u32;

    /// The index space for memories includes respective imports declared in the same module. The
    /// indices of these imports precede the indices of other definitions in the same index space.
    pub type Mem = u32;

    /// The index space for globals includes respective imports declared in the same module. The
    /// indices of these imports precede the indices of other definitions in the same index space.
    pub type Global = u32;

    /// Element indices reference element segments
    pub type Elem = u32;

    /// Data indices reference data segments.
    pub type Data = u32;

    /// The index space for locals is only accessible inside a function and includes the parameters
    /// of that function, which precede the local variables.
    pub type Local = u32;

    /// Label indices reference structured control instructions inside an instruction sequence.
    pub type Label = u32;
}

/// The types component of a module defines a vector of function types. All function types used in
/// a module must be defined in this component. They are referenced by type indices.
/// [Spec](https://webassembly.github.io/spec/core/syntax/modules.html#types)
pub type Type = FunctionType;

/// The funcs component of a module defines a vector of functions.
/// [Spec](https://webassembly.github.io/spec/core/syntax/modules.html#types)
#[derive(Debug, Default, Clone)]
pub struct Function {
    /// The type of a function declares its signature by reference to a type defined in the module.
    /// The parameters of the function are referenced through 0-based local indices in the
    /// function's body; they are mutable.
    pub functype: index::Type,

    /// The locals declare a vector of mutable local variables and their types. These variables are
    /// referenced through local indices in the function's body. The index of the first local is
    /// the smallest index not referencing a parameter.
    pub locals: Box<[ValueType]>,

    /// The body is an instruction sequence that upon termination must produce a stack matching the
    /// function type's result type.
    pub body: Box<Expr>,
}

/// The tables component of a module defines a vector of tables described by their table type.
/// A table is a vector of opaque values of a particular reference type.
/// [Spec](https://webassembly.github.io/spec/core/syntax/modules.html#tables)
pub type Table = TableType;

/// The mems component of a module defines a vector of linear memories (or memories for short) as
/// described by their memory type.
/// [Spec](https://webassembly.github.io/spec/core/syntax/modules.html#memories)
///
/// A memory is a vector of raw uninterpreted bytes. The min size in the limits of the memory
/// type specifies the initial size of that memory, while its max, if present, restricts the size
/// to which it can grow later. Both are in units of page size.  
pub type Memory = MemType;

/// The globals component of a module defines a vector of global variables (or globals for short).  
/// [Spec](https://webassembly.github.io/spec/core/syntax/modules.html#globals)
///
/// Each global stores a single value of the given global type. Its type also specifies whether a
/// global is immutable or mutable. Moreover, each global is initialized with an init value given
/// by a constant initializer expression.
#[derive(Debug, Clone)]
pub struct Global {
    pub typ: GlobalType,
    pub init: Box<Expr>,
}

/// The elems component of a module defines a vector of element segments. [Spec]
///
/// The initial contents of a table is uninitialized. Element segments can be used to initialize a
/// subrange of a table from a static vector of elements.  
///
/// The elems component of a module defines a vector of element segments. Each element segment
/// defines an reference type and a corresponding list of constant element expressions.
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/modules.html#element-segments
#[derive(Debug, Clone)]
pub struct Elem {
    pub typ: RefType,
    pub init: Box<[Box<Expr>]>,
    pub mode: ElemMode,
}

/// A mode identifying the element segment as passive, active, or declarative.
/// [Spec](https://webassembly.github.io/spec/core/syntax/modules.html#element-segments)
///
/// Element segments have a mode that identifies them as either passive, active, or declarative.
///
/// A passive element segment's elements can be copied to a table using the table.init instruction.
/// An active element segment copies its elements into a table during instantiation, as specified
/// by a table index and a constant expression defining an offset into that table. A declarative
/// element segment is not available at runtime but merely serves to forward-declare references
/// that are formed in code with instructions like ref.func.
#[derive(Debug, Clone)]
pub enum ElemMode {
    Passive,
    Active {
        idx: index::Table,
        offset: Box<Expr>,
    },
    Declarative,
}

/// The datas component of a module defines a vector of data segments. [Spec][Spec]
///
/// The initial contents of a memory are zero bytes. Data segments can be used to initialize a
/// range of memory from a static vector of bytes.
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/modules.html#data-segments
#[derive(Debug, Clone)]
pub struct Data {
    pub init: Box<Expr>,
    pub mode: DataMode,
}

/// A mode identifying the data segment as passive or active.
/// [Spec](https://webassembly.github.io/spec/core/syntax/modules.html#data-segments)
///
/// Like element segments, data segments have a mode that identifies them as either passive or
/// active. A passive data segment's contents can be copied into a memory using the memory.init
/// instruction. An active data segment copies its contents into a memory during instantiation, as
/// specified by a memory index and a constant expression defining an offset into that memory.
#[derive(Debug, Clone)]
pub enum DataMode {
    Passive,
    Active { idx: index::Mem, offset: Box<Expr> },
}

/// A function to automatic invoke when the module is instantiated. [Spec]
///
/// The start component of a module declares the function index of a start function that is
/// automatically invoked when the module is instantiated, after tables and memories have been
/// initialized.  
///
/// [Spec]: https://webassembly.github.io/spec/core/syntax/modules.html#start-function
pub type Start = index::Func;

/// A set of exports that become accessible to the host environment once the module has been
/// instantiated.
/// [Spec](https://webassembly.github.io/spec/core/syntax/modules.html#exports)
#[derive(Debug, Clone)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

/// The type of exportable definition defined by an [Export].
/// [Spec](https://webassembly.github.io/spec/core/syntax/modules.html#exports)
///
/// Exportable definitions are functions, tables, memories, and globals, which are referenced
/// through a respective descriptor.
#[derive(Debug, Clone)]
pub enum ExportDesc {
    Func(index::Type),
    Table(index::Type),
    Memory(index::Mem),
    Global(index::Global),
}

/// A set of imports that are required for instantiation of the module.
/// [Spec](https://webassembly.github.io/spec/core/syntax/modules.html#imports)
///
/// Each import is labeled by a two-level name space, consisting of a module name
/// and a name for an entity within that module.
#[derive(Debug, Clone)]
pub struct Import {
    pub module_name: String,
    pub name: String,
    pub desc: ImportDesc,
}

/// The type of importable definition defined by an [Import].
/// [Spec](https://webassembly.github.io/spec/core/syntax/modules.html#exports)
///
/// Importable definitions are functions, tables, memories, and globals. Each import is specified
/// by a descriptor with a respective type that a definition provided during instantiation is
/// required to match.  Every import defines an index in the respective index space. In each index
/// space, the indices of imports go before the first index of any definition contained in the
/// module itself.
#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(index::Type),
    Table(TableType),
    Memory(MemType),
    Global(GlobalType),
}
