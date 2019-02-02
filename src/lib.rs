#![deny(missing_docs)]

/*!
This crate reexports my most commonly used parts of the Rust standard library.
It also adds a few functions, structs, traits and macros for convenience in common patterns:

* [`Adapter`](struct.Adapter.html) Wraps a reference to a string representation of some type
* [`BoolMap`](trait.BoolMap.html) A trait intended for `bool`s that allows one-line constuction of `Option`s
* [`variant!`](macro.variant.html) Maps an enum to an option for use with `Iterator::filter_map`
* [`order`](order/index.html) Functions for fully ordering `PartialOrd` types
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
    str::FromStr,
    thread::{self, JoinHandle},
    vec::IntoIter,
};

/**
Maps an enum to an option for use with `Iterator::filter_map`

# Syntax
```ignore
variant!( pattern = input => output )
```

# Example
```
use kai::*;

enum Foo {
    Bar(bool),
    Baz(i32),
}

let bar = Foo::Bar(true);
let baz = Foo::Baz(5);

assert_eq!(None,    variant!(Foo::Baz(i) = bar => i));
assert_eq!(Some(5), variant!(Foo::Baz(i) = baz => i));

let foos = vec![
    Foo::Baz(5),
    Foo::Bar(false),
    Foo::Baz(2),
];

// Turns this
let ints: Vec<i32> = foos
    .iter()
    .filter_map(|foo| if let Foo::Baz(i) = foo {
        Some(i)
    } else {
        None
    })
    .cloned()
    .collect();

// Into this
let ints: Vec<i32> = foos
    .iter()
    .filter_map(|foo| variant!(Foo::Baz(i) = foo => i))
    .cloned()
    .collect();

assert_eq!(vec![5, 2], ints);
```
*/
#[macro_export]
macro_rules! variant {
    ($pattern:pat = $input:expr => $output:expr) => {
        if let $pattern = $input {
            Some($output)
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
pub trait BoolMap {
    /// Map to an optional value
    fn map<T>(self, value: T) -> Option<T>;
    /// Map to an optional value using a function
    fn map_with<T, F>(self, f: F) -> Option<T>
    where
        F: FnMut() -> T;
}

impl<B> BoolMap for B
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

/**
Wraps a reference to a string representation of some type

The string can be accessed as if it were the type.
An `Adapter` can be made for any type that implements `FromStr` and `ToString`.
An `Adapter` must be dropped before the string can be accessed again.

# Example
```
use kai::*;

// A `Vec` of number strings
let mut nums: Vec<String> = vec![
    "4".into(),
    "1".into(),
    "-1".into(),
];

// Iterate over `Adapters` that wrap the number strings
// The `Adapter`s can be modified as if they are numbers
for mut n in nums.iter_mut().filter_map(|s| Adapter::<i32>::from(s).ok()) {
    *n += 2;
    *n *= 2;
}

assert_eq!(
    vec!["12".to_string(), "6".into(), "2".into()],
    nums,
);
```

*/
pub struct Adapter<'a, T>
where
    T: FromStr + ToString,
{
    string: &'a mut String,
    temp: T,
}

impl<'a, T> Adapter<'a, T>
where
    T: FromStr + ToString,
{
    /// Create a new `Adapter` from a `String`
    pub fn from(string: &'a mut String) -> Result<Adapter<'a, T>, T::Err> {
        string.parse().map(move |temp| Adapter { string, temp })
    }
}

impl<'a, T> Deref for Adapter<'a, T>
where
    T: FromStr + ToString,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.temp
    }
}

impl<'a, T> DerefMut for Adapter<'a, T>
where
    T: FromStr + ToString,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.temp
    }
}

impl<'a, T> Drop for Adapter<'a, T>
where
    T: FromStr + ToString,
{
    fn drop(&mut self) {
        *self.string = self.temp.to_string()
    }
}

impl<'a, T> Debug for Adapter<'a, T>
where
    T: FromStr + ToString,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        <String as Debug>::fmt(self.string, f)
    }
}

impl<'a, T> Display for Adapter<'a, T>
where
    T: FromStr + ToString,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        <String as Display>::fmt(self.string, f)
    }
}

impl<'a, T> AsRef<T> for Adapter<'a, T>
where
    T: FromStr + ToString,
{
    fn as_ref(&self) -> &T {
        &self.temp
    }
}

impl<'a, T> std::borrow::Borrow<T> for Adapter<'a, T>
where
    T: FromStr + ToString,
{
    fn borrow(&self) -> &T {
        &self.temp
    }
}

/**
Functions for fully ordering `PartialOrd` types

These functions are intended for use with certain standard library
functions that take a `Fn(&T, &T) -> Ordering` to order items, such as
`Iterator::max_by`, `Iterator::min_by`, and `Vec::sort_by`.

# Example
```
use kai::*;

let mut v: Vec<f32> = vec![1.0, 0.1, -4.1, 5.2];

v.sort_by(order::or_less);
let max = *v.iter().max_by(order::or_greater).unwrap();

assert_eq!(
    vec![-4.1, 0.1, 1.0, 5.2],
    v
);
assert_eq!(5.2, max);
```
*/
pub mod order {
    use std::cmp::Ordering;
    /// Order and use `Ordering::Less` as a default
    pub fn or_less<T>(a: &T, b: &T) -> Ordering
    where
        T: PartialOrd,
    {
        a.partial_cmp(&b).unwrap_or(Ordering::Less)
    }
    /// Order and use `Ordering::Greater` as a default
    pub fn or_greater<T>(a: &T, b: &T) -> Ordering
    where
        T: PartialOrd,
    {
        a.partial_cmp(&b).unwrap_or(Ordering::Greater)
    }
    /// Order and use `Ordering::Equal` as a default
    pub fn or_equal<T>(a: &T, b: &T) -> Ordering
    where
        T: PartialOrd,
    {
        a.partial_cmp(&b).unwrap_or(Ordering::Equal)
    }
}
