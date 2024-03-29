use {
    crate::{
        log_tag::Tag,
        runtime::error::{Result, TrapKind},
        syntax::{
            types::{Limits, MemType},
            MemoryField, Validated,
        },
    },
    std::ops::Range,
    wrausmt_common::{
        logger::{Logger, PrintLogger},
        true_or::TrueOr,
    },
};

/// A memory instance is the runtime representation of a linear memory.
/// [Spec][Spec]
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
    logger:     PrintLogger,
    pub limits: Limits,
    pub data:   Vec<u8>,
}

const PAGE_SIZE: usize = 65536;

impl MemInstance {
    /// Create a new [MemInstance] for the provided [MemType].
    /// As per the [Spec][Spec], the meory is initialized to `n` pages of `0`s,
    /// where `n` is the lower value of the
    /// [Limits] in the provided [MemType].
    ///
    /// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#memory-instances
    pub fn new(memtype: MemType<Validated>) -> MemInstance {
        let data = vec![0u8; memtype.limits.lower as usize * PAGE_SIZE];
        MemInstance {
            logger: PrintLogger,
            limits: memtype.limits,
            data,
        }
    }

    pub fn new_ast(memfield: MemoryField<Validated>) -> Result<MemInstance> {
        let memtype = memfield.memtype;
        let data = vec![0u8; memtype.limits.lower as usize * PAGE_SIZE];
        Ok(MemInstance {
            logger: PrintLogger,
            limits: memtype.limits,
            data,
        })
    }

    pub fn size(&self) -> usize {
        self.data.len() / PAGE_SIZE
    }

    pub fn grow(&mut self, pgs: u32) -> Option<u32> {
        if pgs as usize > i32::MAX as usize / PAGE_SIZE {
            return None;
        }

        let old_size_in_pages = self.data.len() / PAGE_SIZE;

        if let Some(upper) = self.limits.upper {
            if old_size_in_pages as u32 + pgs > upper {
                return None;
            }
        }

        let newsize = self.data.len() + (pgs as usize * PAGE_SIZE);
        self.data.resize(newsize, 0);

        Some(old_size_in_pages as u32)
    }

    fn offset(&self, o: usize, b: usize, n: usize) -> Result<Range<usize>> {
        let i = o + b;
        let end = i + n;
        self.logger
            .log(Tag::Mem, || format!("READ {} IN {}", i, self.data.len()));
        (end <= self.data.len())
            .true_or(TrapKind::OutOfBoundsMemoryAccess(end, self.data.len()))?;
        Ok(i..end)
    }

    pub fn read(&self, o: usize, b: usize, n: usize) -> Result<&[u8]> {
        let range = self.offset(o, b, n)?;
        Ok(&self.data[range])
    }

    pub fn write(&mut self, o: usize, b: usize, bs: &[u8]) -> Result<()> {
        let range = self.offset(o, b, bs.len())?;
        self.data[range].clone_from_slice(bs);
        Ok(())
    }

    pub fn copy_within(&mut self, src: usize, dst: usize, count: usize) -> Result<()> {
        (src + count <= self.data.len()).true_or(TrapKind::OutOfBoundsMemoryAccess(src, count))?;
        (dst + count <= self.data.len()).true_or(TrapKind::OutOfBoundsMemoryAccess(dst, count))?;
        self.data.copy_within(src..src + count, dst);
        Ok(())
    }
}
