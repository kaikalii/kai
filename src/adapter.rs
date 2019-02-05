use super::*;

/**
Wraps a reference to a `String` representation of some type

The `String` can be accessed as if it were the type.
An `Adapter` can be made for any type that implements `FromStr` and `Display`.
An `Adapter` must be dropped before the `String` can be accessed again.

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
    T: FromStr + Display,
{
    string: &'a mut String,
    temp: T,
}

impl<'a, T> Adapter<'a, T>
where
    T: FromStr + Display,
{
    /// Create a new `Adapter` from a `String`
    pub fn from(string: &'a mut String) -> Result<Adapter<'a, T>, T::Err> {
        string.parse().map(move |temp| Adapter { string, temp })
    }
    /**
    Force a drop, returning ownership to the string

    This function only needs to be called if you want access to the string
    before the `Adapter` would normally be dropped.
    */
    pub fn finish(self) {}
}

impl<'a, T> Deref for Adapter<'a, T>
where
    T: FromStr + Display,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.temp
    }
}

impl<'a, T> DerefMut for Adapter<'a, T>
where
    T: FromStr + Display,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.temp
    }
}

impl<'a, T> Drop for Adapter<'a, T>
where
    T: FromStr + Display,
{
    fn drop(&mut self) {
        *self.string = self.temp.to_string()
    }
}

impl<'a, T> Debug for Adapter<'a, T>
where
    T: FromStr + Display + Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        <T as Debug>::fmt(&self.temp, f)
    }
}

impl<'a, T> Display for Adapter<'a, T>
where
    T: FromStr + Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        <T as Display>::fmt(&self.temp, f)
    }
}

impl<'a, T> AsRef<T> for Adapter<'a, T>
where
    T: FromStr + Display,
{
    fn as_ref(&self) -> &T {
        &self.temp
    }
}

impl<'a, T> std::borrow::Borrow<T> for Adapter<'a, T>
where
    T: FromStr + Display,
{
    fn borrow(&self) -> &T {
        &self.temp
    }
}
