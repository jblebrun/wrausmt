use crate::types::MemType;

/// A memory instance is the runtime representation of a linear memory. [Spec][Spec]
///
/// It records its type and holds a vector of bytes.
///
/// The length of the vector always is a multiple of the WebAssembly page size,
/// which is defined to be the constant 65536 – abbreviated 64Ki
///
/// The bytes can be mutated through memory instructions, the execution of an
/// active data segment, or by external means provided by the embedder.
/// It is an invariant of the semantics that the length of the byte vector,
/// divided by page size, never exceeds the maximum size of memtype, if present.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#memory-instances
#[derive(Default, Debug)]
pub struct MemInstance {
    pub memtype: MemType,
    pub data: Vec<u8>,
}

const PAGE_SIZE: usize = 65536;

impl MemInstance {
    /// Create a new [MemInstance] for the provided [MemType].
    /// As per the [Spec][Spec], the meory is initialized to `n` pages of `0`s,
    /// where `n` is the lower value of the [Limits] in the provided [MemType].
    pub fn new(memtype: MemType) -> MemInstance {
        let data = vec![0u8; memtype.limits.lower as usize * PAGE_SIZE];
        MemInstance { memtype, data }
    }
}
