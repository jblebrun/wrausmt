/// An data instance is the runtime representation of a data segment.
/// [Spec][Spec]
///
/// It holds a vector of bytes.
///
/// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#data-instances
#[derive(Default, Debug)]
pub struct DataInstance {
    pub bytes: Box<[u8]>,
}
