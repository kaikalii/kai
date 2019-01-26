#![deny(missing_docs)]

/*!
This crate reexports my most commonly used parts of the Rust standard library.
It also adds a few traits and macros for convenience in common patterns.
*/

pub use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    error::Error,
    f32, f64,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs::{self, File},
    io::{BufRead, Read, Result as IoResult, Write},
    iter,
    ops::{Deref, DerefMut, Index, IndexMut},
    path::{Path, PathBuf},
    thread::{self, JoinHandle},
    vec::IntoIter,
};

/**
Maps an enum to an option for use with `Iterator::filter_map`

# Example
```
use kai::*;

let strs = vec!["1", "dog", "2.3"];
let floats: Vec<f32> = strs
    .into_iter()
    .map(|s| s.parse::<f32>())
    .filter_map(|f| variant_filter!(Ok(f) = f => f))
    .collect();
assert_eq!(vec![1.0, 2.3], floats);
```
*/
#[macro_export]
macro_rules! variant_filter {
    ($pattern:pat = $expr:expr => $res:expr) => {
        if let $pattern = $expr {
            Some($res)
        } else {
            None
        }
    };
}

/**
A trait intended for `bool`s that allows one-line constuction of `Option`s

# Example
```
use kai::*;

let condition = true;

// Turn this:
let s = if condition {
    Some(String::new())
} else {
    None
};

// Into this:
condition.map_with(String::new);
```
*/
pub trait Map {
    /// Map to an optional value
    fn map<T>(self, value: T) -> Option<T>;
    /// Map to an optional value using a function
    fn map_with<T, F>(self, f: F) -> Option<T>
    where
        F: FnMut() -> T;
}

impl<B> Map for B
where
    B: Into<bool>,
{
    fn map<T>(self, value: T) -> Option<T> {
        if self.into() {
            Some(value)
        } else {
            None
        }
    }
    fn map_with<T, F>(self, mut f: F) -> Option<T>
    where
        F: FnMut() -> T,
    {
        if self.into() {
            Some(f())
        } else {
            None
        }
    }
}
