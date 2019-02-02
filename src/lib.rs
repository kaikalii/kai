#![deny(missing_docs)]

/*!
# Description

This library contains my person Rust prelude and utilities.

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
* [`promote_then`](fn.promote_then.html) Temporarily gain access an immutable reference as mutable

### Traits
* [`BoolMap`](trait.BoolMap.html) Maps `bool`s to `Option`s in one line

### Structs
* [`Adapter`](struct.Adapter.html) Wraps a reference to a string representation of some type

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

transparent_mod!(adapter, hybrid_iter);

pub use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    error::Error,
    f32::consts::PI as PI32,
    f64::consts::PI as PI64,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs::{self, File},
    io::{BufRead, Read, Result as IoResult, Write},
    iter,
    ops::{Deref, DerefMut, Index, IndexMut},
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
Temporarily gain access an immutable reference as mutable

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
