/// ResolvedState is used to track whether or not the symbolic indices in the module have been
/// resolved into the proper numeric values. This needs to happen in a second pass after the
/// initial parse, since index usage may occur before the index has been defined.
///
pub trait ResolvedState: std::fmt::Debug {}

/// A module parameterized by the [Resolved] type will have undergone index resolution,  and type
/// use resolution, and should be safe to compile further.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Resolved {}
impl ResolvedState for Resolved {}

/// A module parameterized by the [IndicesResolved] type will have undergone index resolution, but
/// not type use resolution.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IndicesResolved {}
impl ResolvedState for IndicesResolved {}

/// A module parameterized by the [Resolved] type will have undergone index resolution, and must be
/// compiled before it can be used by the runtime.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Unresolved {}
impl ResolvedState for Unresolved {}

pub trait IndexSpace {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FuncIndex {}
impl IndexSpace for FuncIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TypeIndex {}
impl IndexSpace for TypeIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableIndex {}
impl IndexSpace for TableIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GlobalIndex {}
impl IndexSpace for GlobalIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MemoryIndex {}
impl IndexSpace for MemoryIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataIndex {}
impl IndexSpace for DataIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElemIndex {}
impl IndexSpace for ElemIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LocalIndex {}
impl IndexSpace for LocalIndex {}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LabelIndex {}
impl IndexSpace for LabelIndex {}
