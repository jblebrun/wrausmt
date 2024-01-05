use {
    crate::{
        logger::{Logger, PrintLogger, Tag},
        runtime::error::{Result, RuntimeErrorKind, TrapKind},
        syntax::{types::MemType, MemoryField},
    },
    std::ops::Range,
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
    logger:      PrintLogger,
    pub memtype: MemType,
    pub data:    Vec<u8>,
}

const PAGE_SIZE: usize = 65536;

impl MemInstance {
    /// Create a new [MemInstance] for the provided [MemType].
    /// As per the [Spec][Spec], the meory is initialized to `n` pages of `0`s,
    /// where `n` is the lower value of the
    /// [Limits][crate::syntax::types::Limits] in the provided [MemType].
    ///
    /// [Spec]: https://webassembly.github.io/spec/core/exec/runtime.html#memory-instances
    pub fn new(memtype: MemType) -> MemInstance {
        let data = vec![0u8; memtype.limits.lower as usize * PAGE_SIZE];
        MemInstance {
            logger: PrintLogger,
            memtype,
            data,
        }
    }

    pub fn new_ast(memfield: MemoryField) -> Result<MemInstance> {
        let memtype = memfield.memtype;
        if memtype.limits.lower > 65536 {
            Err(RuntimeErrorKind::ValidationError("Memory too large".to_string()).into())
        } else {
            let data = vec![0u8; memtype.limits.lower as usize * PAGE_SIZE];
            Ok(MemInstance {
                logger: PrintLogger,
                memtype,
                data,
            })
        }
    }

    pub fn size(&self) -> usize {
        self.data.len() / PAGE_SIZE
    }

    pub fn grow(&mut self, pgs: u32) -> Option<u32> {
        if pgs as usize > i32::MAX as usize / PAGE_SIZE {
            return None;
        }

        let old_size_in_pages = self.data.len() / PAGE_SIZE;

        if let Some(upper) = self.memtype.limits.upper {
            if old_size_in_pages as u32 + pgs > upper {
                return None;
            }
        }

        let newsize = self.data.len() + (pgs as usize * PAGE_SIZE);
        self.data.resize(newsize, 0);

        Some(old_size_in_pages as u32)
    }

    fn offset(&self, o: usize, b: usize, n: usize) -> Result<Range<usize>> {
        let i = o as u64 + b as u64;
        self.logger
            .log(Tag::Mem, || format!("READ {} IN {}", i, self.data.len()));
        if (i + n as u64) > self.data.len() as u64 {
            return Err(TrapKind::OutOfBoundsMemoryAccess(
                (i + n as u64) as usize,
                self.data.len(),
            )
            .into());
        }
        let i = i as usize;
        Ok(i..i + n)
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
        (src + count <= self.data.len())
            .then_some(())
            .ok_or(TrapKind::OutOfBoundsMemoryAccess(src, count))?;
        (dst + count <= self.data.len())
            .then_some(())
            .ok_or(TrapKind::OutOfBoundsMemoryAccess(dst, count))?;
        self.data.copy_within(src..src + count, dst);
        Ok(())
    }
}
