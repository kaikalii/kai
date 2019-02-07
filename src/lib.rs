#![deny(missing_docs)]

/*!
# Description

This library contains my personal Rust prelude and utilities.

# Design goals

* Remove the hassle of having to add `use` statements for very common standard library types
* Reduce the amount of code that actually has to be written
* Alleviate common Rust pain points

This library is meant to improve your experience writing Rust no matter what you are writing code for. The patterns it tackles are mostly ones that the average Rust programmer encounters on a daily basis.

# Utilities

I have made some very simple utilities to aid in writing Rust code:

### Functions
* [`order`](order/index.html) Functions for fully ordering `PartialOrd` types
* [`close`](close/index.html) Functions for checking if two floating-point numbers are close enough to be considered equal
* [`promote_then`](fn.promote_then.html) Temporarily gain access to an immutable reference as mutable

### Traits
* [`BoolMap`](trait.BoolMap.html) Maps `bool`s to `Option`s in one line
* [`Bind`](trait.Bind.html) Allows the binding and mutation of a value in a single line
* [`KaiIterator`](trait.KaiIterator.html) Generates my custom iterator adapters

### Structs
* [`Adapter`](struct.Adapter.html) Wraps a reference to a string representation of some type
* [`Swap`](struct.Swap.html) Wrapper that allows consuming transformations on borrowed data

### Types
* [`DynResult`](type.DynResult.html) A dynamic `Result` type
* [`IoResult`](type.IoResult.html) An alias for `io::Result`
* [`FmtResult`](type.FmtResult.html) An alias for `fmt::Result`

### Macros
* [`variant!`](macro.variant.html) Maps an enum to an option for use with `Iterator::filter_map`
* [`transparent_mod!`](macro.transparent_mod.html) Declares transparent external child modules
*/

/**
Declares transparent external child modules

# Example
```ignore
use kai::*;

transparent_mod!(foo, bar, baz);

// Expands to

mod foo;
pub use foo::*;
mod bar;
pub use bar::*;
mod baz;
pub use baz::*;
```
*/
#[macro_export]
macro_rules! transparent_mod {
    ($($mod:ident),*) => {
        $(
            mod $mod;
            pub use $mod::*;
        )*
    };
    ($($mod:ident,)*) => {
        transparent_mod!($($mod),*);
    }
}

transparent_mod!(adapter, swap);

pub use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    error::Error,
    f32::consts::PI as PI32,
    f64::consts::PI as PI64,
    fmt::{Debug, Display, Formatter},
    fs::{self, File},
    io::{stdin, BufRead, Read, Write},
    iter,
    ops::{self, Deref, DerefMut, Index, IndexMut},
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
    sync::{Arc, Mutex},
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
Maps `bool`s to `Option`s in one line

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
let s = condition.map_with(String::new);

assert_eq!(Some(String::new()), s);
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
Allows the binding and mutation of a value in a single line

This is useful when you want a functional interface wrapping a mutable one,
or when you really feel like doing something in one line.

# Example
```
use kai::*;

// Turn this
let mut a = vec![1, 4, 2, 1, 3, 2, 2];
a.sort();
a.dedup();

// Into this
let b = vec![1, 4, 2, 1, 3, 2, 2].bind(|v| v.sort()).bind(Vec::dedup);

assert_eq!(a, b);
```
*/
pub trait Bind: Sized {
    /// Binds the value, mutates it, and returns it
    fn bind<F>(mut self, mut f: F) -> Self
    where
        F: FnMut(&mut Self),
    {
        f(&mut self);
        self
    }
}

impl<T> Bind for T {}

/**
An iterator adaptor created by [`KaiIterator::chain_if`](trait.KaiIterator.html#method.chain_if)
*/
pub enum ChainIf<I, J>
where
    I: IntoIterator,
    J: IntoIterator<Item = I::Item>,
{
    /// The iterator was chained
    Chained(I::IntoIter, J::IntoIter),
    /// The iterator was not chained
    NotChained(I::IntoIter),
}

impl<I, J> Iterator for ChainIf<I, J>
where
    I: IntoIterator,
    J: IntoIterator<Item = I::Item>,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        use ChainIf::*;
        match self {
            Chained(first, second) => first.next().or_else(|| second.next()),
            NotChained(iter) => iter.next(),
        }
    }
}

/**
Generates my custom iterator adapters

For convenience, this is implmented for all types that
implement not just [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html),
but [`IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html) as well.

See methods for usage.
*/
pub trait KaiIterator: IntoIterator + Sized {
    /**
    Chain this iterator with another if the condition is true

    # Example
    ```
    use kai::*;

    let condition = true;

    // Turn this
    let mut v = vec![1, 2, 3];
    if condition {
        v.extend(vec![4, 5, 6])
    }

    // Into this
    let w: Vec<_> = vec![1, 2, 3].chain_if(condition, || vec![4, 5, 6]).collect();

    assert_eq!(v, w);
    ```
    */
    fn chain_if<I, F>(self, condition: bool, mut f: F) -> ChainIf<Self, I>
    where
        I: IntoIterator<Item = Self::Item>,
        F: FnMut() -> I,
    {
        if condition {
            ChainIf::Chained(self.into_iter(), f().into_iter())
        } else {
            ChainIf::NotChained(self.into_iter())
        }
    }
}

impl<I> KaiIterator for I where I: IntoIterator + Sized {}

/**
An dynamic `Result` type
*/
pub type DynResult<T> = Result<T, Box<dyn Error>>;

/**
An alias for `io::Result`
*/
pub type IoResult<T> = std::io::Result<T>;

/**
An alias for `fmt::Result`
*/
pub type FmtResult = std::fmt::Result;

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

/**
Functions for checking if two floating-point numbers are close enough to be considered equal

These functions use the `std::f**::EPSILON` constants to check if two numbers are close
enough for their difference to be the result of rounding errors. I made these primarily to
get clippy off my back about directly comparing floats.
*/
pub mod close {
    /// Check if two `f32`s are close enough to be considered equal
    pub fn f32(a: f32, b: f32) -> bool {
        (a - b).abs() < std::f32::EPSILON
    }
    /// Check if two `&f32`s are close enough to be considered equal
    pub fn f32_ref(a: &f32, b: &f32) -> bool {
        (*a - *b).abs() < std::f32::EPSILON
    }
    /// Check if two `f64`s are close enough to be considered equal
    pub fn f64(a: f64, b: f64) -> bool {
        (a - b).abs() < std::f64::EPSILON
    }
    /// Check if two `&f64`s are close enough to be considered equal
    pub fn f64_ref(a: &f64, b: &f64) -> bool {
        (*a - *b).abs() < std::f64::EPSILON
    }
}

/**
Temporarily gain access to an immutable reference as mutable

This function attempts to make promoting a reference to be mutable slightly less unsafe.
It does this by wrapping access to the mutable reference in a closure, thus bounding
the lifetime. This allows the compiler to reject certain unsafe behaviors and misuse
of the mutable reference. That being said, there are probably still a ton of things
that could go wrong, so this function is still marked `unsafe`. If you are someone who
knows the specific ways that using this function could still cause undefined behvaior,
please let me know by emailing me or opening an issue.

# Example
```
use kai::*;

let v = vec![5];
let x = unsafe { promote_then(&v, |v| v.remove(0)) };
assert!(v.is_empty());
assert_eq!(5, x);
```
*/
pub unsafe fn promote_then<T, R, F>(var: &T, f: F) -> R
where
    F: FnOnce(&mut T) -> R,
{
    f((var as *const T as *mut T).as_mut().unwrap())
}
