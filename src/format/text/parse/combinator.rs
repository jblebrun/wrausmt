use super::error::Result;
use std::io::Read;
use super::Parser;

type ParseFn<S, T> = fn(&mut S) -> Result<Option<T>>;
type ParseGroupFn<S, T> = fn(&mut S) -> Result<Option<Vec<T>>>;

impl<R: Read> Parser<R> {
    /// Attempts to parse a series of items using the provided parse method.
    /// The parse method should return 0 or 1 of the item type.
    /// Returns the results as a vector of items.
    pub fn zero_or_more<T>(&mut self, parse: ParseFn<Self, T>) -> Result<Vec<T>> {
        let mut result: Vec<T> = vec![];
        while let Some(t) = parse(self)? {
            result.push(t);
        }
        Ok(result)
    }

    /// Attempts to parse a series of items using the provided parse method.
    /// The parse method should return 0 or more of the item type.
    /// Returns the results as a flattened vector of items.
    pub fn zero_or_more_groups<T>(&mut self, parse: ParseGroupFn<Self, T>) -> Result<Vec<T>> {
        let mut result: Vec<T> = vec![];
        while let Some(t) = parse(self)? {
            result.extend(t);
        }
        Ok(result)
    }

    /// Returns the first successful parse result from the provided list of 
    /// parse methods, otherwise none.
    pub fn first_of<T>(&mut self, parsers: &[ParseFn<Self,T>]) -> Result<Option<T>> {
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
