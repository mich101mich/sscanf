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
#![cfg_attr(doc_cfg, feature(doc_cfg))]
// TODO: Update format options
// TODO: Talk about regex escaping (JavaScript)
// TODO: Remove chrono
// TODO: Remove error
// TODO: from_scanf docs
// TODO: derive: add docs, add tests, Generics
// TODO: fail tests for type index, format options, derive
// TODO: Add more format options

//! A sscanf (inverse of format!()) Macro based on Regex
//!
//! # sscanf
//! `sscanf` is originally a C-function that takes a String, a format String with placeholders and
//! several Variables (in the Rust version replaced with Types). It then parses the input String,
//! writing the values behind the placeholders into the Variables (Rust: returns a Tuple). `sscanf`
//! can be thought of as reversing a call to `format!()`:
//! ```
//! # use sscanf::scanf;
//! // format: takes format string and values, returns String
//! let s = format!("Hello {}{}!", "World", 5);
//! assert_eq!(s, "Hello World5!");
//!
//! // scanf: takes String, format string and types, returns Tuple
//! let parsed = scanf!(s, "Hello {}{}!", str, usize);
//!
//! // parsed is Result<(&str, usize), sscanf::Error>
//! assert_eq!(parsed.unwrap(), ("World", 5));
//!
//! // alternative syntax:
//! let parsed2 = scanf!(s, "Hello {str}{usize}!");
//! assert_eq!(parsed2.unwrap(), ("World", 5));
//! ```
//! `scanf!()` takes a format String like `format!()`, but doesn't write
//! the values into the placeholders (`{}`), but extracts the values at those `{}` into the return
//! Tuple.
//!
//! If matching the format string failed, an Error is returned:
//! ```
//! # use sscanf::scanf;
//! let s = "Text that doesn't match the format string";
//! let parsed = scanf!(s, "Hello {}_{}!", str, usize);
//! let error_message = parsed.unwrap_err().to_string();
//! assert_eq!(error_message, "scanf: The regex did not match the input");
//! ```
//!
//! Note that the original C-function and this Crate are called sscanf, which is the technically
//! correct version in this context. `scanf` (with one `s`) is a similar C-function that reads a
//! console input instead of taking a String parameter. The macro itself is called `scanf!()`
//! because that is shorter, can be pronounced without sounding too weird and nobody uses the stdin
//! version anyway.
//!
//! **Types in Placeholders:**
//!
//! The types can either be given as a separate parameter after the format string, or directly
//! inside of the `{}` placeholder.  
//! The first allows for autocomplete while typing, syntax highlighting and better compiler errors
//! generated by scanf in case that the wrong types are given.  
//! The second imitates the [Rust format!() behavior since 1.58](https://blog.rust-lang.org/2022/01/13/Rust-1.58.0.html#captured-identifiers-in-format-strings).
//! This option does not allow any paths (like `std::string::String`) or any other form that might
//! contain a `:`, since `:` marks the start of [Format Options](#format-options).
//! It also gives [worse compiler errors](#a-note-on-compiler-errors) when using stable Rust.
//!
//! More examples of the capabilities of [`scanf`]:
//! ```
//! # use sscanf::scanf;
//! let input = "<x=3, y=-6, z=6>";
//! let parsed = scanf!(input, "<x={i32}, y={i32}, z={i32}>");
//! assert_eq!(parsed.unwrap(), (3, -6, 6));
//!
//! let input = "Move to N36E21";
//! let parsed = scanf!(input, "Move to {char}{usize}{char}{usize}");
//! assert_eq!(parsed.unwrap(), ('N', 36, 'E', 21));
//!
//! let input = "Escape literal { } as {{ and }}";
//! let parsed = scanf!(input, "Escape literal {{ }} as {{{{ and }}}}");
//! assert_eq!(parsed.unwrap(), ());
//!
//! let input = "A Sentence with Spaces. Another Sentence.";
//! // str and String do the same, but String clones from the input string
//! // to take ownership instead of borrowing.
//! let (a, b) = scanf!(input, "{String}. {String}.").unwrap();
//! assert_eq!(a, "A Sentence with Spaces");
//! assert_eq!(b, "Another Sentence");
//!
//! // Number format options
//! let input = "0xab01  0o127  101010  1Z";
//! let parsed = scanf!(input, "{usize:x}  {i32:o}  {u8:b}  {u32:r36}");
//! let (a, b, c, d) = parsed.unwrap();
//! assert_eq!(a, 0xab01);     // Hexadecimal
//! assert_eq!(b, 0o127);      // Octal
//! assert_eq!(c, 0b101010);   // Binary
//!
//! assert_eq!(d, 71);         // any radix (r36 = Radix 36)
//! assert_eq!(d, u32::from_str_radix("1Z", 36).unwrap());
//!
//! let input = "color: #D4AF37";
//! // Number types take their size into account, and hexadecimal u8 can
//! // have at most 2 digits => only possible match is 2 digits each.
//! let (r, g, b) = scanf!(input, "color: #{u8:x}{u8:x}{u8:x}").unwrap();
//! assert_eq!((r, g, b), (0xD4, 0xAF, 0x37));
//! ```
//! The input in this case is a `&'static str`, but in can be `String`, `&str`, `&String`, ...
//! Basically anything with [`Deref<Target=str>`](https://doc.rust-lang.org/std/ops/trait.Deref.html).
//! and without taking Ownership. This also means that the input might need to outlive the
//! `scanf!()` call, because the [`Error`](enum.Error.html)
//! type borrows from it and using [`str`](trait.RegexRepresentation.html#impl-RegexRepresentation-for-str)
//! returns a slice from the input.
//!
//! The parsing part of this macro has very few limitations, since it replaces the `{}` with a
//! Regular Expression ([`regex`](https://docs.rs/regex)) that corresponds to that type.
//! For example:
//! - `char` is just one Character (regex `"."`)
//! - `str` is any sequence of Characters (regex `".+?"`)
//! - Numbers are any sequence of digits (regex `"[-+]?\d+"`)
//!
//! And so on. The actual implementation for numbers tries to take the size of the Type into
//! account and some other details, but that is the gist of the parsing.
//!
//! This means that any sequence of replacements is possible as long as the Regex finds a
//! combination that works. In the `char, usize, char, usize` example above it manages to assign
//! the `N` and `E` to the `char`s because they cannot be matched by the `usize`s.
//!
//! # Format Options
//! All Options are inside `'{'` `'}'` and after a `:`. Literal `'{'` or `'}'` inside of a Format
//! Option are escaped as `'\{'` instead of `'{{'` to avoid ambiguity.
//!
//! Procedural macro don't have any reliable type info and can only compare types by name. This means
//! that the number options below only work with a literal type like "`i32`", **NO** Paths (~~`std::i32`~~)
//! or Wrappers (~~`struct Wrapper(i32);`~~) or Aliases (~~`type Alias = i32;`~~). **ONLY** `i32`,
//! `usize`, `u16`, ...
//!
//! | config                      | description                | possible types |
//! | --------------------------- | -------------------------- | -------------- |
//! | `{:/` _\<regex>_ `/}`       | custom regex               | any            |
//! | `{:x}`                      | hexadecimal numbers        | numbers        |
//! | `{:o}`                      | octal numbers              | numbers        |
//! | `{:b}`                      | binary numbers             | numbers        |
//! | `{:r2}` - `{:r36}`          | radix 2 - radix 36 numbers | numbers        |
//!
//! **Custom Regex:**
//!
//! - `{:/.../}`: Match according to the [`Regex`](https://docs.rs/regex) between the `/` `/`
//!
//! For example:
//! ```
//! # use sscanf::scanf;
//! let input = "random Text";
//! let parsed = scanf!(input, "{str:/[^m]+/}{str}");
//!
//! // regex  [^m]+  matches anything that isn't an 'm'
//! // => stops at the 'm' in 'random'
//! assert_eq!(parsed.unwrap(), ("rando", "m Text"));
//! ```
//!
//! Note: If you use any unescaped ( ) in your regex, you have to prevent them from forming
//! a capture group by adding a `?:` at the beginning: `{:/..(..)../}` becomes `{:/..(?:..)../}`.
//!
//! As mentioned previously, `'{'` `'}'` have to be escaped with a `'\'`. This means that:
//! - `"{"` or `"}"` would give a compiler error
//! - `"\{"` or `"\}"` lead to a `"{"` or `"}"` inside of the regex
//!   - curly brackets have a special meaning in regex as counted repetition
//! - `"\\{"` or `"\\}"` would give a compiler error
//!   - first `'\'` escapes the second one, leaving just the brackets
//! - `"\\\{"` or `"\\\}"` lead to a `"\{"` or `"\}"` inside of the regex
//!   - the first `'\'` escapes the second one, leading to a literal `'\'` in the regex. the third
//!     escapes the curly bracket as in the second case
//!   - needed in order to have the regex match an actual curly bracket
//!
//! Note that this is only the case if you are using raw strings for formats, regular strings require
//! escaping `'\'`, so this would double the number of `'\\'`.
//!
//! Works with non-`String` types too:
//! ```
//! # use sscanf::scanf;
//! let input = "Match 4 digits: 123456";
//! let parsed = scanf!(input, r"Match 4 digits: {usize:/\d{4}/}{usize}");
//!                            // raw string r"" to write \d instead of \\d
//!
//! // regex  \d{4}  matches exactly 4 digits
//! assert_eq!(parsed.unwrap(), (1234, 56));
//! ```
//! Note that changing the regex of a non-`String` type might cause that type's [`FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html)
//! to fail
//!
//! **Number Options:**
//!
//! Only work on primitive number types (`u8`, ..., `u128`, `i8`, ..., `i128`, `usize`, `isize`).
//! - `x`: hexadecimal Number (Digits 0-9 and a-f or A-F, optional Prefix `0x`)
//! - `o`: octal Number (Digits 0-7, optional Prefix `0o`)
//! - `b`: binary Number (Digits 0-1, optional Prefix `0b`)
//! - `r2` - `r36`: any radix Number (no prefix)
//!
//! # Custom Types
//!
//! [`scanf`] works with most primitive Types from `std` as well as `String` by default. The
//! full list can be seen here: [Implementations of `RegexRepresentation`](./trait.RegexRepresentation.html#foreign-impls).
//!
//! More Types can easily be added, as long as they implement [`FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html) for the parsing
//! and [`RegexRepresentation`] for `scanf` to obtain the Regex of the Type:
//! ```
//! # use sscanf::scanf;
//! # #[derive(Debug, PartialEq)]
//! struct TimeStamp {
//!     year: usize, month: u8, day: u8,
//!     hour: u8, minute: u8,
//! }
//! impl sscanf::RegexRepresentation for TimeStamp {
//!     /// Matches "[year-month-day hour:minute]"
//!     const REGEX: &'static str = r"\[\d\d\d\d-\d\d-\d\d \d\d:\d\d\]";
//! }
//! impl std::str::FromStr for TimeStamp {
//!     // ...
//! #   type Err = std::num::ParseIntError;
//! #   fn from_str(s: &str) -> Result<Self, Self::Err> {
//! #       // if you read this: Stop stalking my Code, and yes I know this is lazy. shut up.
//! #       let res = s.split(&['-', ' ', ':', '[', ']'][..]).collect::<Vec<_>>();
//! #       Ok(TimeStamp {
//! #           year: res[1].parse::<usize>()?,
//! #           month: res[2].parse::<u8>()?,
//! #           day: res[3].parse::<u8>()?,
//! #           hour: res[4].parse::<u8>()?,
//! #           minute: res[5].parse::<u8>()?,
//! #       })
//! #   }
//! }
//!
//! let input = "[1518-10-08 23:51] Guard #751 begins shift";
//! let parsed = scanf!(input, "{TimeStamp} Guard #{usize} begins shift");
//! assert_eq!(parsed.unwrap(), (TimeStamp{
//!     year: 1518, month: 10, day: 8,
//!     hour: 23, minute: 51
//! }, 751));
//! ```
//!
//! Implementing `RegexRepresentation` isn't _strictly_ necessary if you **always** supply a custom
//! Regex when using the type by using the `{:/.../}` format option, but this tends to make your code
//! less readable.
//!
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
//! # use sscanf::scanf;
//! scanf!("", "Some Text {}{}{} and stuff", usize);
//! ```
//! ```text
//! error: Missing Type for given '{}' Placeholder
//!   |
//! 4 | scanf!("", "Some Text {}{}{} and stuff", usize);
//!   |                         ^^
//! ```
//! But on stable, you are limited to only pointing at the entire format string:
//! ```text
//! error: Missing Type for given '{}' Placeholder:
//!        At "Some Text {}{}{} and stuff"
//!                        ^^
//!   |
//! 4 | scanf!("", "Some Text {}{}{} and stuff", usize);
//!   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! ```
//! The current workaround is to replicate that behavior in the error message
//! itself. The alternative is to use `cargo +nightly check` to see the better errors
//! whenever something goes wrong, or setting your Editor plugin to check with nightly.
//!
//! This does _**not**_ influence the functionality in any way. This Crate works entirely on stable
//! with no drawbacks in functionality or performance. The only difference is the compiler errors
//! that you get while writing format strings.

/// A Macro to parse a String based on a format-String, similar to sscanf in C
///
/// ## Signature
/// ```ignore
/// scanf!(input: impl Deref<Target=str> + 'input, format: <literal>, Type...) -> Result<(Type...), Error<'input>>
/// ```
///
/// ## Parameters
/// * `input`: The String to parse. Can be anything that implements [`Deref<Target=str>`](std::ops::Deref)
/// * `format`: A literal string. No const or static allowed, just like with [`format!()`](std::format).
/// * `Type...`: The Types to parse. Can be any type that implements [`RegexRepresentation`] and [`FromStr`](std::str::FromStr).
///
/// Returns: A [`Result`](std::result::Result) with the parsed Types or an [`Error`](Error) if the parsing failed.
///
/// The format string _has_ to be a string literal (with some form of `"` on either side),
/// because it is parsed by the procedural macro at compile time and checks if all the types
/// and placeholders are matched. This is not possible from inside a Variable or even a `const
/// &str` somewhere else.
///
/// Placeholders within the format string are marked with `{}`. Any `{` or `}` that should not be
/// treated as placeholders need to be escaped by writing `{{` or `}}`. For every placeholder there
/// has to be a Type name inside the `{}` or exactly one Type in the parameters after the format
/// string.
///
/// Any additional formatting options are placed behind a `:`. For a list of options, see
/// the [crate root documentation](index.html#format-options).
///
/// Note the lifetime `'input` on Error. This is the lifetime of the input string, which is borrowed
/// by the Error. If one of the types is `str`, it will be returned as `&'input str`.
///
/// ## Examples
/// More examples can be seen in the crate root documentation.
/// ```
/// use sscanf::scanf;
///
/// let input = "<x=3, y=-6, z=6>";
/// let parsed = scanf!(input, "<x={}, y={}, z={}>", i32, i32, i32); // types in parameters
/// assert_eq!(parsed.unwrap(), (3, -6, 6));
///
/// let input = "Goto N36E21";
/// let parsed = scanf!(input, "Goto {char}{usize}{char}{usize}"); // types in placeholders
/// assert_eq!(parsed.unwrap(), ('N', 36, 'E', 21));
///
/// let input = "4-5 t: ftttttrvts";
/// let parsed = scanf!(input, "{usize}-{usize} {}: {str}", char); // mixed types (discouraged)
/// assert_eq!(parsed.unwrap(), (4, 5, 't', "ftttttrvts"));
/// ```
pub use sscanf_macro::scanf;

/// Same as [`scanf`], but returns the Regex without running it. Useful for Debugging or Efficiency.
///
/// ## Signature
/// ```ignore
/// scanf_get_regex!(format: <literal>, Type...) -> &'static Regex
/// ```
///
/// ## Parameters
/// * `format`: A literal string. No const or static allowed, just like with [`format!()`](std::format).
/// * `Type...`: The Types to parse. Can be any type that implements [`RegexRepresentation`] and [`FromStr`](std::str::FromStr).
///
/// Returns: A reference to the generated [`Regex`](regex::Regex).
///
/// The Placeholders can be obtained by capturing the Regex and using the 1-based index of the Group.
///
/// ## Examples
/// ```
/// use sscanf::scanf_get_regex;
/// let input = "Test 5 -2";
/// let regex = scanf_get_regex!("Test {} {}", usize, i32);
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
pub use sscanf_macro::scanf_get_regex;

/// Same as [`scanf`], but allows use of Regex in the format String.
///
/// Signature and Parameters are the same as [`scanf`].
///
/// ## Examples
/// ```
/// use sscanf::scanf_unescaped;
/// let input = "5.0SOME_RANDOM_TEXT3";
/// let output = scanf_unescaped!(input, "{f32}.*{usize}");
/// assert_eq!(output.unwrap(), (5.0, 3));
/// ```
///
/// The basic [`scanf`] would escape the `.` and `*`and match against the literal Characters,
/// as one would expect from a Text matcher:
/// ```
/// use sscanf::scanf;
/// let input = "5.0SOME_RANDOM_TEXT3";
/// let output = scanf!(input, "{f32}.*{usize}");
/// assert!(output.is_err()); // does not match
///
/// let input2 = "5.0.*3";
/// let output2 = scanf!(input2, "{f32}.*{usize}"); // regular scanf is unaffected by special characters
/// assert_eq!(output2.unwrap(), (5.0, 3));
/// ```
///
/// Note that the `{{` and `}}` Escaping for literal `{` and `}` is still in place:
/// ```
/// use sscanf::scanf_unescaped;
/// let input = "5.0 } aaaaaa 3";
/// let output = scanf_unescaped!(input, r"{} \}} a{{6}} {}", f32, usize);
///   // in regular Regex this would be   ...  \} a{6} ...
/// assert_eq!(output.unwrap(), (5.0, 3));
/// ```
///
/// Also Note: `^` and `$` are added automatically to the start and end.
pub use sscanf_macro::scanf_unescaped;

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
