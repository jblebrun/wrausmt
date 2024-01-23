use {
    super::{error::Result, pctx, Parser},
    std::io::Read,
};

type ParseFn<S, T> = fn(&mut S) -> Result<Option<T>>;
type ParseGroupFn<S, T> = fn(&mut S) -> Result<Option<Vec<T>>>;

/// The result type for [zero_or_more_groups] containing the flattened result,
/// as well as a count of how many groups were encountered.
pub struct Groups<T> {
    pub result: Vec<T>,
    pub count:  u32,
}

impl<R: Read> Parser<R> {
    /// Attempts to parse a series of items using the provided parse method.
    /// The parse method should return 0 or 1 of the item type.
    /// Returns the results as a vector of items.
    pub fn zero_or_more<T>(&mut self, parse: ParseFn<Self, T>) -> Result<Vec<T>> {
        pctx!(self, "zero or more");
        let mut result: Vec<T> = vec![];
        while let Some(t) = parse(self)? {
            result.push(t);
        }
        Ok(result)
    }

    /// Attempts to parse a series of items using the provided parse method.
    /// The parse method should return 0 or more of the item type.
    /// Returns the results as a flattened vector of items.
    pub fn zero_or_more_groups<T>(&mut self, parse: ParseGroupFn<Self, T>) -> Result<Groups<T>> {
        pctx!(self, "zero or more groups");
        let mut result: Vec<T> = vec![];
        let mut count = 0u32;
        while let Some(t) = parse(self)? {
            count += 1;
            result.extend(t);
        }
        Ok(Groups { result, count })
    }

    /// Attempts to parse a series of items using the provided parse method.
    /// The parse method should return 0 or more of the item type.
    /// Accepts an existing Vec of T, which it will extend.
    pub fn zero_or_more_groups_extend<T>(
        &mut self,
        parse: ParseGroupFn<Self, T>,
        result: &mut Vec<T>,
    ) -> Result<()> {
        pctx!(self, "zero or more groups extend");
        while let Some(t) = parse(self)? {
            result.extend(t);
        }
        Ok(())
    }

    /// Returns the first successful parse result from the provided list of
    /// parse methods, otherwise none.
    pub fn first_of<T>(&mut self, parsers: &[ParseFn<Self, T>]) -> Result<Option<T>> {
        pctx!(self, "first of");
        for parse in parsers {
            match parse(self) {
                Err(e) => return Err(e),
                Ok(Some(t)) => return Ok(Some(t)),
                Ok(None) => (),
            }
        }
        Ok(None)
    }
}
