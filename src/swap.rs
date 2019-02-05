use super::*;

/**
Wrapper that allows consuming transformations on borrowed data

This is useful when you have a mutable interface wrapping a functional one.

# Example
```
use kai::*;

struct Foo {
    v: Swap<Vec<i32>>
}

impl Foo {
    fn keep_even(&mut self) {
        self.v.hold(|v| v.into_iter().filter(|n| n % 2 == 0).collect());
    }
}

let mut foo = Foo { v: vec![1, 2, 3, 4, 5].into() };
foo.keep_even();
assert_eq!(vec![2, 4], *foo.v);
```
*/
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Swap<T>(Option<T>);

impl<T> Swap<T> {
    /// Create a new `Swap`
    pub fn new(inner: T) -> Self {
        Swap::from(inner)
    }
    /// Take the inner value, transform it, and put it back in place
    pub fn hold<F>(&mut self, f: F)
    where
        F: FnOnce(T) -> T,
    {
        let res = f(self.0.take().unwrap());
        self.0 = Some(res);
    }
    /// Take the inner value
    pub fn into_inner(self) -> T {
        self.0.unwrap()
    }
}

impl<T> Deref for Swap<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T> DerefMut for Swap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

impl<T> From<T> for Swap<T> {
    fn from(inner: T) -> Self {
        Swap(Some(inner))
    }
}

impl<T> Default for Swap<T>
where
    T: Default,
{
    fn default() -> Self {
        Swap::from(T::default())
    }
}

impl<T> Debug for Swap<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        <T as Debug>::fmt(self.0.as_ref().unwrap(), f)
    }
}

impl<T> Display for Swap<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        <T as Display>::fmt(self.0.as_ref().unwrap(), f)
    }
}

impl<T> AsRef<T> for Swap<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref().unwrap()
    }
}

impl<T> std::borrow::Borrow<T> for Swap<T> {
    fn borrow(&self) -> &T {
        self.0.as_ref().unwrap()
    }
}
