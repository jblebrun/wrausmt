use std::fmt;
use super::super::error::{ErrorFrom, Error};

#[derive(Debug)]
pub struct ArgumentCountError {
    expected: usize,
    got: usize,
}

impl ArgumentCountError {
    pub fn new(
        expected: usize, 
        got: usize
    ) -> ArgumentCountError { 
        ArgumentCountError { expected, got } 
    }
} 

impl std::error::Error for ArgumentCountError {}

impl fmt::Display for ArgumentCountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected {} args but got {}", self.expected, self.got)
    }
}


