#![deny(
    missing_docs,
    missing_debug_implementations,
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
#![doc = include_str!("../Readme.md")]
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

mod macros;
pub use macros::*;

mod regex_representation;
pub use regex_representation::*;

mod from_scanf;
pub use from_scanf::*;

mod types;
pub use types::*;

pub mod errors;
#[doc(inline)]
pub use errors::Error;
#[doc(hidden)]
pub use errors::{FromScanfFailedError, FromStrFailedError}; // for backwards compatibility

#[doc = include_str!("../Changelog.md")]
pub mod changelog {}

#[doc(hidden)]
pub use const_format;
#[doc(hidden)]
pub use lazy_static;
#[doc(hidden)]
pub use regex;

#[allow(unused_imports)]
use std::str::FromStr; // for links in the documentation
