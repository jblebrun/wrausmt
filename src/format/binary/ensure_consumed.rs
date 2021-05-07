use super::error::{BinaryParseError, Result};

/// A convenience method for take to assert that its contents have been fully consumed.
pub trait EnsureConsumed {
    fn limit(&self) -> u64;
    fn ensure_consumed(&self) -> Result<()> {
        let remaining = self.limit();
        if remaining > 0 {
            Err(BinaryParseError::ExtraSectionBytes(remaining))
        } else {
            Ok(())
        }
    }
}

impl<T> EnsureConsumed for std::io::Take<T> {
    fn limit(&self) -> u64 {
        Self::limit(self)
    }
}
