#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    rustdoc::missing_doc_code_examples,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::bare_urls
)]
// TODO: Update format options
// TODO: Talk about regex escaping (JavaScript)
// TODO: Remove chrono
// TODO: Remove error
// TODO: from_sscanf docs
// TODO: derive: add docs, add tests, Generics
// TODO: fail tests for type index, format options, derive, :#r16
// TODO: Add more format options

#![doc = include_str!("../README.md")]
//! # A Note on Compiler Errors
//!
//! Errors in the format string would ideally point to the exact position in the string that
//! caused the error. This is already the case if you compile/check with nightly, but not on
//! stable, or at least until Rust Issue [`#54725`](https://github.com/rust-lang/rust/issues/54725)
//! is far enough to allow for [`this method`](https://doc.rust-lang.org/proc_macro/struct.Literal.html#method.subspan)
//! to be called from stable.
//!
//! Compiler Errors on nightly currently look like this:
//! ```compile_fail
//! # use sscanf::sscanf;
//! sscanf!("", "Too many placeholders: {}{}{}", usize);
//! ```
//! ```text
//! error: more placeholders than types provided
//!   |
//! 4 | sscanf!("", "Too many placeholders: {}{}{}", usize);
//!   |                                       ^^
//! ```
//! But on stable, you are limited to only pointing at the entire format string:
//! ```text
//! 4 | sscanf!("", "Too many placeholders: {}{}{}", usize);
//!   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! ```
//! The current workaround is to replicate that behavior in the error message
//! itself:
//! ```text
//! error: more placeholders than types provided:
//!        At "Too many placeholders: {}{}{}"
//!                                     ^^
//!   |
//! 4 | sscanf!("", "Too many placeholders: {}{}{}", usize);
//!   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! ```
//!
//! The alternative is to use `cargo +nightly check` to see the better errors
//! whenever something goes wrong, or setting your Editor plugin to check with nightly.
//!
//! This does _**not**_ influence the functionality in any way. This Crate works entirely on stable
//! with no drawbacks in functionality or performance. The only difference is the compiler errors
//! that you get while writing format strings.

/// A Macro to parse a string based on a format-string, similar to sscanf in C
///
/// ## Signature
/// ```ignore
/// sscanf!(input: impl Deref<Target=str>, format: <literal>, Type...) -> Result<(Type...), sscanf::Error>
/// ```
///
/// ## Parameters
/// * `input`: The string to parse. Can be anything that implements [`Deref<Target=str>`](std::ops::Deref)
///   (e.g. `&str`, `String`, `Cow<str>`, etc. See examples below). Note that `sscanf` does not take
///   ownership of the input.
/// * `format`: A literal string. No const or static allowed, just like with [`format!()`](std::format).
/// * `Type...`: The types to parse. See [Custom Types](index.html#custom-types) for more information.
///
/// ## Return Value
/// A [`Result`](std::result::Result) with a tuple of the parsed types or a [`sscanf::Error`](crate::Error).
/// Note that an error usually indicates that the input didn't match the format string, making the
/// returned [`Result`](std::result::Result) functionally equivalent to an [`Option`](std::option::Option),
/// and most applications should treat it that way. An error is only useful when debugging
/// custom implementations of [`FromStr`](std::str::FromStr) or [`FromScanf`](crate::FromScanf).
/// See [`sscanf::Error`](crate::Error) for more information.
///
/// ## Details
/// The format string _has_ to be a string literal (with some form of `"` on either side),
/// because it is parsed by the procedural macro at compile time and checks if all the types
/// and placeholders are matched. This is not possible from inside a variable or even a `const
/// &str` somewhere else.
///
/// Placeholders within the format string are marked with `{}`. Any `'{'` or `'}'` that should not be
/// treated as placeholders need to be escaped by writing `'{{'` or `'}}'`. For every placeholder there
/// has to be a type name inside the `{}` or exactly one type in the parameters after the format
/// string. Types can be referenced by indices in the placeholder, similar to [`format!()`](std::fmt).
///
/// Any additional formatting options are placed behind a `:`. For a list of options, see
/// the [crate root documentation](index.html#format-options).
///
/// ## Examples
/// A few examples for possible inputs:
/// ```
/// # use sscanf::sscanf;
/// let input = "5"; // &str
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
///
/// let input = String::from("5"); // String
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
///
/// // does not work because it creates a temporary value
/// // assert_eq!(sscanf!(String::from("5"), "{usize}").unwrap(), 5);
///
/// let input = &input; // &String
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
/// assert_eq!(sscanf!(input.as_str(), "{usize}").unwrap(), 5);
///
/// let input = std::borrow::Cow::from("5"); // Cow<str>
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
///
/// let input = std::rc::Rc::from("5"); // Rc<str>
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
///
/// // and many more
/// ```
///
/// More Examples can be seen in the crate root documentation.
pub use sscanf_macro::sscanf;

#[doc(hidden)]
pub use sscanf_macro::sscanf as scanf;

/// Same as [`sscanf`], but returns the regex without running it. Useful for debugging or efficiency.
///
/// ## Signature
/// ```ignore
/// sscanf_get_regex!(format: <literal>, Type...) -> &'static Regex
/// ```
///
/// ## Parameters
/// * `format`: A literal string. No const or static allowed, just like with [`format!()`](std::format).
/// * `Type...`: The types to parse. See [Custom Types](index.html#custom-types) for more information.
///
/// Returns: A reference to the generated [`Regex`](regex::Regex).
///
/// The Placeholders can be obtained by capturing the Regex and using the 1-based index of the Group.
///
/// ## Examples
/// ```
/// use sscanf::sscanf_get_regex;
/// let input = "Test 5 -2";
/// let regex = sscanf_get_regex!("Test {usize} {i32}");
/// assert_eq!(regex.as_str(), r"^Test (\+?\d{1,20}) ([-+]?\d{1,10})$");
///
/// let output = regex.captures(input);
/// assert!(output.is_some());
/// let output = output.unwrap();
///
/// let capture_5 = output.get(1);
/// assert!(capture_5.is_some());
/// assert_eq!(capture_5.unwrap().as_str(), "5");
///
/// let capture_negative_2 = output.get(2);
/// assert!(capture_negative_2.is_some());
/// assert_eq!(capture_negative_2.unwrap().as_str(), "-2");
/// ```
pub use sscanf_macro::sscanf_get_regex;

#[doc(hidden)]
pub use sscanf_macro::sscanf_get_regex as scanf_get_regex;

/// Same as [`sscanf`], but allows use of Regex in the format String.
///
/// Signature and Parameters are the same as [`sscanf`].
///
/// ## Examples
/// ```
/// use sscanf::sscanf_unescaped;
/// let input = "5.0SOME_RANDOM_TEXT3";
/// let output = sscanf_unescaped!(input, "{f32}.*?{usize}"); // .*? matches anything
/// assert_eq!(output.unwrap(), (5.0, 3));
/// ```
///
/// The basic [`sscanf`] would escape the `.`, `*` and `?`and match against the literal Characters,
/// as one would expect from a Text matcher:
/// ```
/// use sscanf::sscanf;
/// let input = "5.0SOME_RANDOM_TEXT3";
/// let output = sscanf!(input, "{f32}.*{usize}");
/// assert!(output.is_err()); // does not match
///
/// let input2 = "5.0.*3";
/// let output2 = sscanf!(input2, "{f32}.*{usize}"); // regular sscanf is unaffected by special characters
/// assert_eq!(output2.unwrap(), (5.0, 3));
/// ```
///
/// Note that the `{{` and `}}` escaping for literal `{` and `}` is still required.
///
/// Also note that `^` and `$` are automatically added to the start and end.
pub use sscanf_macro::sscanf_unescaped;

#[doc(hidden)]
pub use sscanf_macro::sscanf_unescaped as scanf_unescaped;

/// TODO: Add documentation
pub use sscanf_macro::FromScanf;

mod regex_representation;
pub use regex_representation::*;

mod from_scanf;
pub use from_scanf::*;

mod types;
pub use types::*;

mod error;
pub use error::*;

#[doc(hidden)]
pub use const_format;
#[doc(hidden)]
pub use lazy_static;
#[doc(hidden)]
pub use regex;

#[allow(unused_imports)]
use std::str::FromStr; // for links in the documentation
