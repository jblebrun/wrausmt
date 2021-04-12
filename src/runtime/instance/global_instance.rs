use crate::runtime::Value;
use crate::types::ValueType;

/// A global instance is the runtime representation of a global variable.
/// [Spec][Spec]
///
/// It records its type and holds an individual value.
///
/// The value of mutable globals can be mutated through variable instructions or
/// by external means provided by the embedder.
///
/// It is an invariant of the semantics that the value has a type equal to the
/// value type of globaltype.
#[allow(dead_code)]
pub struct GlobalInstance {
    pub typ: ValueType,
    pub val: Value,
}
