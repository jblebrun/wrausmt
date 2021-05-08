use std::fmt::Display;
use std::{borrow::Borrow, collections::HashSet, hash::Hash};

pub trait Logger {
    fn log<S: Borrow<str> + Eq + Hash + Display, F>(&self, tag: S, msg: F)
    where
        F: Fn() -> String;
}

#[derive(Debug, Clone)]
pub struct PrintLogger {
    tags: HashSet<String>,
}

impl Default for PrintLogger {
    fn default() -> Self {
        let mut tags = HashSet::default();
        tags.insert("ENTER".to_owned());
        Self { tags }
    }
}

impl Logger for PrintLogger {
    fn log<S: Borrow<str> + Eq + Hash + Display, F>(&self, tag: S, msg: F)
    where
        F: Fn() -> String,
    {
        if self.tags.contains(tag.borrow()) {
            let msg = msg();
            println!("[{}] {}", tag, msg)
        }
    }
}
