/// ResolvedState is used to track whether or not the symbolic indices in the
/// module have been resolved into the proper numeric values. This needs to
/// happen in a second pass after the initial parse, since index usage may occur
/// before the index has been defined.

macro_rules! marker {
    (
        $(#[$($attrss:tt)*])*
        $n:ident: $t:ty
    ) => {
        $(#[$($attrss)*])*
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct $n {}
        impl $t for $n {}
    };
}

pub trait ResolvedState: std::fmt::Debug {}
marker!(
    /// A module parameterized by the [Resolved] type will have undergone index
    /// resolution,  and type use resolution, and should be safe to compile further.
    Resolved: ResolvedState
);
marker!(
    /// A module parameterized by the [IndicesResolved] type will have undergone
    /// index resolution, but not type use resolution.
    IndicesResolved: ResolvedState
);
marker!(
    /// A module parameterized by the [Unresolved] type will have undergone index
    /// resolution, and must be compiled before it can be used by the runtime.
    Unresolved: ResolvedState
);

/// A marker trait to describe the resource that an index refers to.
pub trait IndexSpace {}
marker!(FuncIndex: IndexSpace);
marker!(TypeIndex: IndexSpace);
marker!(TableIndex: IndexSpace);
marker!(GlobalIndex: IndexSpace);
marker!(MemoryIndex: IndexSpace);
marker!(DataIndex: IndexSpace);
marker!(ElemIndex: IndexSpace);
marker!(LocalIndex: IndexSpace);
marker!(LabelIndex: IndexSpace);
