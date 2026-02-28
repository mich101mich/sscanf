#![deny(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    // rustdoc::missing_doc_code_examples, // can be re-enabled when developing, but not useful as a strict rule
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_html_tags,
    rustdoc::bare_urls,
    rustdoc::redundant_explicit_links,
    rustdoc::unescaped_backticks
)]
//
// set of clippy::pedantic lints that I disagree with
#![allow(
    clippy::wildcard_imports, // glob import > importing 20 items
    clippy::enum_glob_use,
    clippy::items_after_statements // if an item is only used locally, define it where it is needed
)]
//
#![doc = include_str!("../Readme.md")]
//! # Compiler errors
//!
//! Ideally, errors in the format string point to the exact position in the string that caused the error. This already
//! works on nightly, but not on stable - at least until Rust Issue
//! [`#54725`](https://github.com/rust-lang/rust/issues/54725) enables calling
//! [`Literal::subspan`](https://doc.rust-lang.org/proc_macro/struct.Literal.html#method.subspan) from stable.
//!
//! Errors on nightly currently look like this:
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
//! On stable, diagnostics can only point at the entire format string:
//! ```text
//! 4 | sscanf!("", "Too many placeholders: {}{}{}", usize);
//!   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! ```
//! The current workaround is to replicate that behavior inside the error message:
//! ```text
//! error: more placeholders than types provided:
//!        At "Too many placeholders: {}{}{}"
//!                                     ^^
//!   |
//! 4 | sscanf!("", "Too many placeholders: {}{}{}", usize);
//!   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! ```
//!
//! Alternatively, use `cargo +nightly check` to see these better errors, or configure your editor to check with
//! nightly.
//!
//! This does _**not**_ affect functionality. This crate works entirely on stable with no performance or
//! feature drawbacks. The only difference is the quality of compiler errors while writing format strings.

mod from_scanf;
mod macros;
mod parser;
pub use from_scanf::*;
pub use macros::*;
pub use parser::*;

pub mod advanced;

#[doc = include_str!("../Changelog.md")]
pub mod changelog {}

/// Parses the given input string into a value of type `T`.
///
/// This is equivalent to `sscanf!(input, "{T}")`.
///
/// This function can be used when [`FromScanf`] was implemented/derived for `T` in such a way that it can parse the
/// entire input string without any additional format string.
///
/// Note that it is rather inefficient to call this function multiple times with the same type `T`, since the parser
/// has to be re-constructed each time. If you need to parse multiple values of the same type, consider using
/// [`Parser::new`] to create a parser once and re-use it multiple times.
pub fn parse<'input, T: FromScanf<'input>>(input: &'input str) -> Option<T> {
    Parser::<T>::new().parse(input)
}

#[cfg(test)]
mod tests {
    /// Utility macro for other tests: Asserts that the given block or statement throws a panic with the given message.
    #[macro_export]
    macro_rules! assert_panic_message_eq {
        ( $block:block, $message:literal $(,)? ) => {
            let Err(error) = std::panic::catch_unwind(move || $block) else {
                panic!("code {} did not panic", stringify!($block));
            };
            if let Some(s) = error.downcast_ref::<&'static str>() {
                assert_eq!(*s, $message);
            } else if let Some(s) = error.downcast_ref::<String>() {
                assert_eq!(s, $message);
            } else {
                panic!("unexpected panic payload: {:?}", error);
            }
        };
        ( $expression:expr, $message:literal $(,)? ) => {
            assert_panic_message_eq!(
                {
                    $expression; // avoid problems with lifetimes by not returning the value
                },
                $message
            );
        };
        ( $statement:stmt, $message:literal $(,)? ) => {
            assert_panic_message_eq!({ $statement }, $message);
        };
    }
}
