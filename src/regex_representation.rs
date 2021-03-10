/// A Trait used by `scanf` to obtain the Regex of a Type
///
/// Has one associated Constant: `REGEX`, which should be set to a regular Expression.
/// Implement this trait for a Type that you want to be parsed using scanf.
///
/// The Regular Expression should match the string representation as exactly as possible.
/// Any incorrect matches might be caught in the from_str parsing, but that might cause this
/// regex to take characters that could have been matched by other placeholders, leading to
/// unexpected parsing failures. Also: Since `scanf` only returns an `Option` it will just say
/// `None` whether the regex matching failed or the parsing failed, so you should avoid parsing
/// failures by writing a proper regex as much as possible.
///
/// ## Example
/// Let's say we want to add a Fraction parser
/// ```
/// # #[derive(Debug, PartialEq)]
/// struct Fraction(isize, usize);
/// ```
/// Which can be obtained from any string of the kind `±X/Y` or just `X`
/// ```
/// # #[derive(Debug, PartialEq)]
/// # struct Fraction(isize, usize);
/// impl sscanf::RegexRepresentation for Fraction {
///     /// matches an optional '-' or '+' followed by a number.
///     /// possibly with a '/' and another Number
///     const REGEX: &'static str = r"[-+]?\d+(/\d+)?";
/// }
/// impl std::str::FromStr for Fraction {
///     type Err = std::num::ParseIntError;
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         let mut iter = s.split('/');
///         let num = iter.next().unwrap().parse::<isize>()?;
///         let mut denom = 1;
///         if let Some(d) = iter.next() {
///             denom = d.parse::<usize>()?;
///         }
///         Ok(Fraction(num, denom))
///     }
/// }
/// ```
/// Now we can use this `Fraction` struct in `scanf`:
/// ```
/// # #[derive(Debug, PartialEq)]
/// # struct Fraction(isize, usize);
/// # impl sscanf::RegexRepresentation for Fraction {
/// #     const REGEX: &'static str = r"[-+]?\d+(/\d+)?";
/// # }
/// # impl std::str::FromStr for Fraction {
/// #     type Err = std::num::ParseIntError;
/// #     fn from_str(s: &str) -> Result<Self, Self::Err> {
/// #         let mut iter = s.split('/');
/// #         let num = iter.next().unwrap().parse::<isize>()?;
/// #         let mut denom = 1;
/// #         if let Some(d) = iter.next() {
/// #             denom = d.parse::<usize>()?;
/// #         }
/// #         Ok(Fraction(num, denom))
/// #     }
/// # }
/// use sscanf::scanf;
///
/// let output = scanf!("2/5", "{}", Fraction);
/// assert_eq!(output, Some(Fraction(2, 5)));
///
/// let output = scanf!("-25/3", "{}", Fraction);
/// assert_eq!(output, Some(Fraction(-25, 3)));
///
/// let output = scanf!("8", "{}", Fraction);
/// assert_eq!(output, Some(Fraction(8, 1)));
///
/// let output = scanf!("6e/3", "{}", Fraction);
/// assert_eq!(output, None);
///
/// let output = scanf!("6/-3", "{}", Fraction);
/// assert_eq!(output, None); // only first number can be negative
///
/// let output = scanf!("6/3", "{}", Fraction);
/// assert_eq!(output, Some(Fraction(6, 3)));
/// ```
pub trait RegexRepresentation {
    /// A regular Expression that exactly matches any String representation of the implementing Type
    const REGEX: &'static str;
}

macro_rules! impl_num {
    (u64: $($ty: ty),+) => {
        $(impl RegexRepresentation for $ty {
            /// Matches any positive number
            ///
            /// The length of this match might not fit into the size of the type
            const REGEX: &'static str = r"\+?\d+";
        })+
    };
    (i64: $($ty: ty),+) => {
        $(impl RegexRepresentation for $ty {
            /// Matches any positive or negative number
            ///
            /// The length of this match might not fit into the size of the type
            const REGEX: &'static str = r"[-+]?\d+";
        })+
    };
    (f64: $($ty: ty),+) => {
        $(impl RegexRepresentation for $ty {
            /// Matches any floating point number
            ///
            /// Does **NOT** support stuff like `inf` `nan` or `3E10`
            const REGEX: &'static str = r"[-+]?\d+\.?\d*";
        })+
    };
}

impl_num!(u64: usize, u64, u128);
impl_num!(i64: isize, i64, i128);
impl_num!(f64: f32, f64);

impl RegexRepresentation for String {
    /// Matches any sequence of Characters
    const REGEX: &'static str = r".+";
}
impl RegexRepresentation for char {
    /// Matches a single Character
    const REGEX: &'static str = r".";
}
impl RegexRepresentation for bool {
    /// Matches `true` or `false`
    const REGEX: &'static str = r"true|false";
}

impl RegexRepresentation for u8 {
    /// Matches a number with up to 3 digits.
    ///
    /// The Number matched by this might be too big for the type
    const REGEX: &'static str = r"\+?\d{1,3}";
}
impl RegexRepresentation for u16 {
    /// Matches a number with up to 5 digits.
    ///
    /// The Number matched by this might be too big for the type
    const REGEX: &'static str = r"\+?\d{1,5}";
}
impl RegexRepresentation for u32 {
    /// Matches a number with up to 10 digits.
    ///
    /// The Number matched by this might be too big for the type
    const REGEX: &'static str = r"\+?\d{1,10}";
}
impl RegexRepresentation for i8 {
    /// Matches a number with possible sign and up to 3 digits.
    ///
    /// The Number matched by this might be too big for the type
    const REGEX: &'static str = r"[-+]?\d{1,3}";
}
impl RegexRepresentation for i16 {
    /// Matches a number with possible sign and up to 5 digits.
    ///
    /// The Number matched by this might be too big for the type
    const REGEX: &'static str = r"[-+]?\d{1,5}";
}
impl RegexRepresentation for i32 {
    /// Matches a number with possible sign and up to 10 digits.
    ///
    /// The Number matched by this might be too big for the type
    const REGEX: &'static str = r"[-+]?\d{1,10}";
}

/// A Wrapper around f32 whose RegexRepresentation also includes special floating point values
/// like `nan`, `inf`, `2.0e5`, ...
///
/// This is not part of the regular f32 parser because having a Number match against Text like with
/// `nan` is usually not desirable:
/// ```
/// # use sscanf::*;
/// let input = "Match a Banana against a number";
/// let output = scanf!(input, "{}{}{}", String, f32, String);
/// // There are no Numbers in input, so expect None
/// assert!(output.is_none());
///
/// let output = scanf!(input, "{}{}{}", String, FullF32, String);
/// // The 'nan' part in "Banana" is parsed as f32::NaN
/// assert!(output.is_some());
/// assert!(output.unwrap().1.is_nan());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct FullF32(pub f32);

impl std::str::FromStr for FullF32 {
    type Err = <f32 as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == "nan" {
            return Ok(FullF32(std::f32::NAN));
        }
        Ok(FullF32(s.parse()?))
    }
}
impl RegexRepresentation for FullF32 {
    /// Matches any floating point number, including `nan`, `inf`, `2.0e5`, ...
    const REGEX: &'static str =
        r"[-+]?\d+\.?\d*|\.\d+|\.\d+[eE][-+]?\d+|[-+]?\d+\.\d+[eE][-+]?\d+|[nN]a[nN]|\-?inf";
}
impl std::ops::Deref for FullF32 {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for FullF32 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A Wrapper around f64 whose RegexRepresentation also includes special floating point values
/// like `nan`, `inf`, `2.0e5`, ...
///
/// See [`FullF32`] for Details
#[derive(Clone, Copy, Debug)]
pub struct FullF64(pub f64);

impl std::str::FromStr for FullF64 {
    type Err = <f64 as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(FullF64(s.parse()?))
    }
}
impl RegexRepresentation for FullF64 {
    /// Matches any floating point number, including `nan`, `inf`, `2.0e5`, ...
    const REGEX: &'static str = FullF32::REGEX;
}
impl std::ops::Deref for FullF64 {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for FullF64 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
