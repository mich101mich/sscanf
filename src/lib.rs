#![deny(
    missing_docs,
    // missing_doc_code_examples,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
//! A Rust crate with a sscanf-style Macro based on Regex
//!
//! TODO: Add more
//! ```
//! use sscanf::scanf;
//!
//! let input = "4-5 t: ftttttrvts";
//! let parsed = scanf!(input, "{}-{} {}: {}", usize, usize, char, String);
//! assert_eq!(parsed, Some((4, 5, 't', String::from("ftttttrvts"))));
//!
//! let input = "<x=3, y=-6, z=6>";
//! let parsed = scanf!(input, "<x={}, y={}, z={}>", i32, i32, i32);
//! assert_eq!(parsed, Some((3, -6, 6)));
//!
//! let input = "Goto N36E21";
//! let parsed = scanf!(input, "Goto {}{}{}{}", char, usize, char, usize);
//! assert_eq!(parsed, Some(('N', 36, 'E', 21)));
//! ```
//!
//! ## Custom Types
//!
//! ```
//! # use sscanf::scanf;
//! # #[derive(Debug, PartialEq)]
//! struct TimeStamp {
//!     year: usize, month: u8, day: u8,
//!     hour: u8, minute: u8,
//! }
//! impl sscanf::RegexRepresentation for TimeStamp {
//!     const REGEX: &'static str = r"\d\d\d\d-\d\d-\d\d \d\d:\d\d";
//! }
//! impl std::str::FromStr for TimeStamp {
//!     // ...
//! #   type Err = std::num::ParseIntError;
//! #   fn from_str(s: &str) -> Result<Self, Self::Err> {
//! #       let res = s.split(&['-', ' ', ':'][..]).collect::<Vec<_>>();
//! #       Ok(TimeStamp {
//! #           year: res[0].parse::<usize>()?,
//! #           month: res[1].parse::<u8>()?,
//! #           day: res[2].parse::<u8>()?,
//! #           hour: res[3].parse::<u8>()?,
//! #           minute: res[4].parse::<u8>()?,
//! #       })
//! #   }
//! }
//!
//! let input = "[1518-10-08 23:51] Guard #751 begins shift";
//! let parsed = scanf!(input, "[{}] Guard #{} begins shift", TimeStamp, usize);
//! assert_eq!(parsed, Some((TimeStamp{
//!     year: 1518, month: 10, day: 8,
//!     hour: 23, minute: 51
//! }, 751)));
//! ```
//!

/// A Macro to parse a String based on a format-String, similar to sscanf in C
///
/// TODO: usage
/// ```
/// use sscanf::scanf;
///
/// let input = "4-5 t: ftttttrvts";
/// let parsed = scanf!(input, "{}-{} {}: {}", usize, usize, char, String);
/// assert_eq!(parsed, Some((4, 5, 't', String::from("ftttttrvts"))));
///
/// let input = "<x=3, y=-6, z=6>";
/// let parsed = scanf!(input, "<x={}, y={}, z={}>", i32, i32, i32);
/// assert_eq!(parsed, Some((3, -6, 6)));
///
/// let input = "Goto N36E21";
/// let parsed = scanf!(input, "Goto {}{}{}{}", char, usize, char, usize);
/// assert_eq!(parsed, Some(('N', 36, 'E', 21)));
/// ```
///
/// TODO: Regex
///
/// TODO: Error Message notice
///
/// Supports any Type that implements both [`FromStr`](::std::str::FromStr) and [`RegexRepresentation`]
pub use sscanf_macro::scanf;

/// Same to [`scanf`], but returns the Regex without running it. Useful for Debugging or Efficiency.
///
/// The Placeholders can be obtained by capturing the Regex and using either Name or index of the Group.
///
/// The Name is always `type_i` where `i` is the index of the type in the `scanf` call, starting at `1`.
///
/// Indices start at `1` as with any Regex, however it is possible that user-defined implementations
/// of [`RegexRepresentation`] create their own Capture Groups and distort the order, so use with caution.
///
/// ```
/// use sscanf::scanf_get_regex;
/// let input = "Test 5 -2";
/// let regex = scanf_get_regex!("Test {} {}", usize, i32);
/// assert_eq!(regex.as_str(), r"^Test (?P<type_1>\+?\d+) (?P<type_2>[-+]?\d{1,10})$");
///
/// let output = regex.captures(input);
/// assert!(output.is_some());
/// let output = output.unwrap();
///
/// let capture_5 = output.name("type_1");
/// assert!(capture_5.is_some());
/// assert_eq!(capture_5.unwrap().as_str(), "5");
/// assert_eq!(capture_5, output.get(1));
///
/// let capture_negative_2 = output.name("type_2");
/// assert!(capture_negative_2.is_some());
/// assert_eq!(capture_negative_2.unwrap().as_str(), "-2");
/// assert_eq!(capture_negative_2, output.get(2));
/// ```
pub use sscanf_macro::scanf_get_regex;

/// Same as [`scanf`], but allows use of Regex in the format String.
///
/// ```
/// use sscanf::scanf_unescaped;
/// let input = "5.0SOME_RANDOM_TEXT3";
/// let output = scanf_unescaped!(input, "{}.*{}", f32, usize);
/// assert_eq!(output, Some((5.0, 3)));
/// ```
///
/// The basic [`scanf`] would escape the `.` and `*`and match against the literal Characters,
/// as one would expect from a Text matcher:
/// ```
/// use sscanf::scanf;
/// let input = "5.0SOME_RANDOM_TEXT3";
/// let output = scanf!(input, "{}.*{}", f32, usize);
/// assert_eq!(output, None); // does not match
///
/// let input2 = "5.0.*3";
/// let output2 = scanf!(input2, "{}.*{}", f32, usize);
/// assert_eq!(output2, Some((5.0, 3)));
/// ```
///
/// Note that the `{{` and `}}` Escaping for literal `{` and `}` is still in Place:
/// ```
/// use sscanf::scanf_unescaped;
/// let input = "5.0 } aaaaaa 3";
/// let output = scanf_unescaped!(input, r"{} \}} a{{6}} {}", f32, usize);
///   // in regular Regex this would be   ...  \} a{6} ...
/// assert_eq!(output, Some((5.0, 3)));
/// ```
///
/// Also Note: `^` and `$` are added automatically to the start and end.
pub use sscanf_macro::scanf_unescaped;

/// A Trait used by [`scanf`] to obtain the Regex of a Type
///
/// Has one associated Constant: `REGEX`, which should be set to a regular Expression.
/// Implement this trait for a Type that you want to be parsed using scanf.
/// TODO: talk about exactness
///
/// ## Example
/// Let's say we want to add a Fraction parser
/// ```
/// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// struct Fraction(isize, usize);
/// ```
/// Which can be obtained from any string of the kind `±X/Y` or just `X`
/// ```
/// # #[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
/// # #[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// re-export of [`const_format::concatcp`](https://docs.rs/const_format/0.2/const_format/macro.concatcp.html) to be used by the proc_macro expansion.
///
pub use const_format::concatcp as const_format;
/// re-export of [`regex::Regex`](https://docs.rs/regex/1.4/regex/struct.Regex.html) to be used by the proc_macro expansion.
///
pub use regex::Regex;

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
    const REGEX: &'static str = r".+";
}
impl RegexRepresentation for char {
    const REGEX: &'static str = r".";
}
impl RegexRepresentation for bool {
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
