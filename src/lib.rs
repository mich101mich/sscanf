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
//! A

/// A Macro to parse a String based on a format-String, similar to sscanf in C
///
/// TODO: usage
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
/// assert_eq!(regex.as_str(), r"^Test (?P<type_1>\+?\d+) (?P<type_2>[-+]?\d+)");
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
/// let capture_2 = output.name("type_2");
/// assert!(capture_2.is_some());
/// assert_eq!(capture_2.unwrap().as_str(), "-2");
/// assert_eq!(capture_2, output.get(2));
/// ```
pub use sscanf_macro::scanf_get_regex;

/// A Trait used by [`scanf`] to obtain the Regex of a Type
///
/// Has one associated Constant: `REGEX`, which should be set to a regular Expression
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
    (u32: $($ty: ty),+) => {
        $(impl RegexRepresentation for $ty {
            const REGEX: &'static str = r"\+?\d+";
        })+
    };
    (i32: $($ty: ty),+) => {
        $(impl RegexRepresentation for $ty {
            const REGEX: &'static str = r"[-+]?\d+";
        })+
    };
    (f32: $($ty: ty),+) => {
        $(impl RegexRepresentation for $ty {
            const REGEX: &'static str = r"[-+]?\d+\.?\d*";
        })+
    };
}

impl_num!(u32: usize, u8, u16, u32, u64, u128);
impl_num!(i32: isize, i8, i16, i32, i64, i128);
impl_num!(f32: f32, f64);

impl RegexRepresentation for String {
    const REGEX: &'static str = r".+";
}
impl RegexRepresentation for char {
    const REGEX: &'static str = r".";
}
impl RegexRepresentation for bool {
    const REGEX: &'static str = r"true|false";
}
