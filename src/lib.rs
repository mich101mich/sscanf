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

//! A sscanf (inverse of format!()) Macro based on Regex
//!
//! # sscanf
//! `sscanf` is originally a C-function that takes a String, a format String with placeholders and
//! several Variables (in the Rust version replaced with Types). It then parses the input String,
//! writing the values behind the placeholders into the Variables (Rust: returns a Tuple). `sscanf`
//! can be thought of as reversing a call to `format!()`:
//! ```
//! // format: takes format string and values, returns String
//! let s = format!("Hello {}_{}!", "World", 5);
//! assert_eq!(s, "Hello World_5!");
//!
//! // scanf: takes String, format string and types, returns Tuple
//! let parsed = sscanf::scanf!(s, "Hello {}_{}!", String, usize);
//! // parsed is Option<(String, usize)>
//! assert_eq!(parsed, Some((String::from("World"), 5)));
//! ```
//! `scanf!()` takes a format String like `format!()`, but doesn't write
//! the values into the placeholders (`{}`), but extracts the values at those `{}` into the return
//! Tuple.
//!
//! If matching the format string failed, `None` is returned:
//! ```
//! # use sscanf::scanf;
//! let s = "Text that doesn't match the format string";
//! let parsed = scanf!(s, "Hello {}_{}!", String, usize);
//! assert_eq!(parsed, None); // No match possible
//! ```
//!
//! Note that the original C-function and this Crate are called sscanf, which is the technically
//! correct version in this context. `scanf` (with one `s`) is a similar C-function that reads a
//! console input instead of taking a String parameter. The macro itself is called `scanf!()`
//! because that is shorter, can be pronounced without sounding too weird and nobody uses the stdin
//! version anyway.
//!
//! More examples of the capabilities of [`scanf`]:
//! ```
//! # use sscanf::scanf;
//! let input = "<x=3, y=-6, z=6>";
//! let parsed = scanf!(input, "<x={}, y={}, z={}>", i32, i32, i32);
//! assert_eq!(parsed, Some((3, -6, 6)));
//!
//! let input = "Move to N36E21";
//! let parsed = scanf!(input, "Move to {}{}{}{}", char, usize, char, usize);
//! assert_eq!(parsed, Some(('N', 36, 'E', 21)));
//!
//! let input = "Escape literal { } as {{ and }}";
//! let parsed = scanf!(input, "Escape literal {{ }} as {{{{ and }}}}");
//! assert_eq!(parsed, Some(()));
//!
//! let input = "A Sentence with Spaces. Another Sentence.";
//! let parsed = scanf!(input, "{}. {}.", String, String);
//! let (a, b) = parsed.unwrap();
//! assert_eq!(a, "A Sentence with Spaces");
//! assert_eq!(b, "Another Sentence");
//!
//! let input = "Formats:  0xab01  0o127  0b101010  1Z";
//! let parsed = scanf!(input, "Formats:  {x}  {o}  {b}  {r36}", usize, i32, u8, u32);
//! let (a, b, c, d) = parsed.unwrap();
//! assert_eq!(a, 0xab01);     // Hex
//! assert_eq!(b, 0o127);      // Octal
//! assert_eq!(c, 0b101010);   // Binary
//!
//! assert_eq!(d, 71);         // any radix (r36 = Radix 36)
//! assert_eq!(d, u32::from_str_radix("1Z", 36).unwrap());
//! ```
//! The input in this case is a `&'static str`, but in can be `String`, `&str`, `&String`, ...
//! Basically anything with [`AsRef<str>`](https://doc.rust-lang.org/std/convert/trait.AsRef.html)
//! and without taking Ownership.
//!
//! The parsing part of this macro has very few limitations, since it replaces the `{}` with a
//! Regular Expression ([`regex`](https://docs.rs/regex)) that corresponds to that type.
//! For example:
//! - `char` is just one Character (regex `"."`)
//! - `String` is any sequence of Characters (regex `".+"`)
//! - Numbers are any sequence of digits (regex `"\d+"`)
//!
//! And so on. The actual implementation for numbers tries to take the size of the Type into
//! account and some other details, but that is the gist of the parsing.
//!
//! This means that any sequence of replacements is possible as long as the Regex finds a
//! combination that works. In the `char, usize, char, usize` example above it manages to assign
//! the `N` and `E` to the `char`s because they cannot be matched by the `usize`s.
//!
//! # Format Options
//! All Options are inside `'{'` `'}'`. Literal `'{'` or `'}'` inside of a Format Option are escaped
//! as `'\{'` instead of `'{{'` to avoid ambiguity.
//!
//! Procedural macro don't have any reliable type info and can only compare types by name. This means
//! that Format Options only work with a literal type like "`i32`", **NO** Paths (~~`std::i32`~~) or
//! Wrappers (~~`struct Wrapper(i32);`~~) or Aliases (~~`type Alias = i32;`~~). **ONLY** `i32`,
//! `usize`, `u16`, ...
//!
//! | config                     | description                | possible types |
//! | -------------------------- | -------------------------- | -------------- |
//! | `{/` _\<regex>_ `/}`       | custom regex               | any            |
//! | `{x}`                      | hexadecimal numbers        | numbers        |
//! | `{o}`                      | octal numbers              | numbers        |
//! | `{b}`                      | binary numbers             | numbers        |
//! | `{r2}`..=`{r36}`           | radix 2 - radix 32 numbers | numbers        |
//! | `{` _\<chrono format>_ `}` | chrono format              | chrono types   |
//!
//! **Custom Regex:**
//!
//! - `{/.../}`: Match according to the [`Regex`](https://docs.rs/regex) between the `/` `/`
//!
//! For example:
//! ```
//! # use sscanf::scanf;
//! let input = "random Text";
//! let parsed = scanf!(input, "{/[^m]+/}{}", String, String);
//!
//! // regex  [^m]+  matches anything that isn't an 'm'
//! // => stops at the 'm' in 'random'
//! assert_eq!(parsed, Some((String::from("rando"), String::from("m Text"))));
//! ```
//!
//! As mentioned above, `'{'` `'}'` have to be escaped with a `'\'`. This means that:
//! - `"{"` or `"}"` would give a compiler error
//! - `"\{"` or `"\}"` lead to a `"{"` or `"}"` inside of the regex
//!   - curly brackets have a special meaning in regex as counted repetition etc.
//! - `"\\{"` or `"\\}"` would give a compiler error
//!   - first `'\'` just escapes the second one, leaving just the brackets
//! - `"\\\{"` or `"\\\}"` lead to a `"\{"` or `"\}"` inside of the regex
//!   - the first `'\'` escapes the second one, leading to a literal `'\'` in the regex. the third
//!     escapes the curly bracket as in the second case
//!   - needed in order to have the regex match an actual curly bracket
//!
//! Works with non-`String` types too:
//! ```
//! # use sscanf::scanf;
//! let input = "Match 4 digits: 123456";
//! let parsed = scanf!(input, r"Match 4 digits: {/\d\{4\}/}{}", usize, usize);
//!                            // raw string (r"") to write \d instead of \\d
//!
//! // regex  \d{4}  matches 4 digits
//! assert_eq!(parsed, Some((1234, 56)));
//! ```
//! Note that changing the regex of a non-`String` type might cause that type's [`FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html)
//! to fail
//!
//! **Number Options:**
//!
//! Only work on primitive number types (`u8`, ..., `u128`, `i8`, ..., `i128`, `usize`, `isize`).
//! - `x`: hexadecimal Number (Digits 0-9 and A-F, optional Prefix `0x`)
//! - `o`: octal Number (Digits 0-7, optional Prefix `0o`)
//! - `b`: binary Number (Digits 0-1, optional Prefix `0b`)
//! - `r2` ..= `r36`: any radix Number (no prefix)
//!
//! **[`chrono`](https://docs.rs/chrono/^0.4/chrono/) integration (Requires `chrono` feature):**
//!
//! The types [`DateTime`](https://docs.rs/chrono/^0.4/chrono/struct.DateTime.html),
//! [`NaiveDate`](https://docs.rs/chrono/^0.4/chrono/naive/struct.NaiveDate.html),
//! [`NaiveTime`](https://docs.rs/chrono/^0.4/chrono/naive/struct.NaiveTime.html),
//! [`NaiveDateTime`](https://docs.rs/chrono/^0.4/chrono/naive/struct.NaiveDateTime.html),
//! [`Utc`](https://docs.rs/chrono/^0.4/chrono/offset/struct.Utc.html) and
//! [`Local`](https://docs.rs/chrono/^0.4/chrono/offset/struct.Local.html) can be used and accept
//! a [Date/Time format string](https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html)
//! inside of the `{` `}`, that will then be used for both the Regex generation and parsing of the
//! type.
//!
//! Using [`DateTime`](https://docs.rs/chrono/^0.4/chrono/struct.DateTime.html) returns a
//! `DateTime<FixedOffset>` and requires the rules and limits that [`DateTime::parse_from_str`](https://docs.rs/chrono/^0.4/chrono/struct.DateTime.html#method.parse_from_str)
//! has.
//!
//! ```
//! # #[cfg(feature = "chrono")]
//! # {
//! # use sscanf::*;
//! use chrono::prelude::*;
//!
//! let input = "10:37:02";
//! let parsed = scanf!(input, "{%H:%M:%S}", NaiveTime);
//! assert_eq!(parsed, Some(NaiveTime::from_hms(10, 37, 2)));
//!
//! let expected = Utc.ymd(2020, 5, 23).and_hms(21, 5, 7);
//!
//! // DateTime<*> directly implements FromStr and doesn't need a config
//! let input = "2020-05-23T21:05:07Z";
//! let parsed = scanf!(input, "{}", DateTime<Utc>);
//! assert_eq!(parsed, Some(expected));
//!
//! let input = "Today is the 23. of May, 2020 at 09:05 pm and 7 seconds.";
//! let parsed = scanf!(input, "Today is the {%d. of %B, %Y at %I:%M %P and %-S} seconds.", Utc);
//! assert_eq!(parsed, Some(expected));
//! # }
//! ```
//!
//! Note: The `chrono` feature needs to be active for this to work, because `chrono` is an optional dependency
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
//! let parsed = scanf!(input, "{} Guard #{} begins shift", TimeStamp, usize);
//! assert_eq!(parsed, Some((TimeStamp{
//!     year: 1518, month: 10, day: 8,
//!     hour: 23, minute: 51
//! }, 751)));
//! ```
//!
//! Implementing `RegexRepresentation` isn't _strictly_ necessary if you **always** supply a custom
//! Regex when using the type by using the `{/.../}` format option, but this tends to make your code
//! less readable.
//!
//! # A Note on Error Messages
//!
//! Errors in the format string would ideally point to the exact position in the string that
//! caused the error. This is already the case if you compile/check with nightly, but not on
//! stable, or at least until Rust Issue [`#54725`](https://github.com/rust-lang/rust/issues/54725)
//! is far enough to allow for [`this method`](https://doc.rust-lang.org/proc_macro/struct.Literal.html#method.subspan)
//! to be called from stable.
//!
//! Error Messages on nightly currently look like this:
//! ```compile_fail
//! # use sscanf::scanf;
//! scanf!("", "Some Text {}{}{} and stuff", usize);
//! ```
//! ```text
//! error: Missing Type for given '{}' Placeholder
//!   |
//! 4 | scanf!("", "Some Text {}{}{} and stuff", usize);
//!   |                                 ^^
//! ```
//! But on stable, you are limited to only pointing at the entire format string:
//! ```text
//! error: Missing Type for given '{}' Placeholder:
//! At "Some Text {}{}{} and stuff"
//!                 ^^
//!   |
//! 4 | scanf!("", "Some Text {}{}{} and stuff", usize);
//!   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! ```
//! The current workaround is to replicate that behavior in the Error Message
//! itself. The alternative is to use `cargo +nightly check` to see the better Errors
//! whenever something goes wrong, or setting your Editor plugin to check with nightly.
//!
//! This does _**not**_ influence the functionality in any way. This Crate works entirely on stable
//! with no drawbacks in functionality or performance. The only difference is the compiler errors
//! that you get while writing format strings.

/// A Macro to parse a String based on a format-String, similar to sscanf in C
///
/// Takes at least two Parameters:
/// - An input string (`String`, `str`, ... as long as it has `AsRef<str>`)
///   - `scanf` does not take Ownership!
/// - A format string literal (see below)
///
/// As well as any number of Types.
///
/// The format string _has_ to be a str literal (with some form of `"` on either side),
/// because it is parsed by the procedural macro at compile time and checks if all the types
/// and placeholders are matched. This is not possible from inside a Variable or even a `const
/// &str` somewhere else.
///
/// Placeholders within the format string are marked with `{}`. Any `{` or `}` that should not be
/// treated as placeholders need to be escaped by writing `{{` or `}}`. For any placeholder there
/// has to be exactly one Type in the parameters after the format string.
///
/// There are currently no additional formatting options inside of the `{}`. This might be added
/// later (no guarantees).
///
/// ## Examples
/// More examples can be seen in the crate root documentation.
/// ```
/// use sscanf::scanf;
///
/// let input = "<x=3, y=-6, z=6>";
/// let parsed = scanf!(input, "<x={}, y={}, z={}>", i32, i32, i32);
/// assert_eq!(parsed, Some((3, -6, 6)));
///
/// let input = "4-5 t: ftttttrvts";
/// let parsed = scanf!(input, "{}-{} {}: {}", usize, usize, char, String);
/// assert_eq!(parsed, Some((4, 5, 't', String::from("ftttttrvts"))));
///
/// let input = "Goto N36E21";
/// let parsed = scanf!(input, "Goto {}{}{}{}", char, usize, char, usize);
/// assert_eq!(parsed, Some(('N', 36, 'E', 21)));
/// ```
pub use sscanf_macro::scanf;

/// Same as [`scanf`], but returns the Regex without running it. Useful for Debugging or Efficiency.
///
/// The Placeholders can be obtained by capturing the Regex and using either Name or index of the Group.
///
/// The Name is always `type_i` where `i` is the index of the type in the `scanf` call, starting at `1`.
///
/// Indices start at `1` as with any Regex, however it is possible that non-std implementations
/// of [`RegexRepresentation`] create their own Capture Groups and distort the order, so use with
/// caution. ([`FullF32`] does this for example)
///
/// ```
/// use sscanf::scanf_get_regex;
/// let input = "Test 5 -2";
/// let regex = scanf_get_regex!("Test {} {}", usize, i32);
/// assert_eq!(regex.as_str(), r"^Test (?P<type_1>\+?\d{1,20}) (?P<type_2>[-+]?\d{1,10})$");
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

mod regex_representation;
pub use regex_representation::*;

mod types;
pub use types::*;

#[doc(hidden)]
pub use const_format;
#[doc(hidden)]
pub use lazy_static;
#[doc(hidden)]
pub use regex;

#[cfg(feature = "chrono")]
#[doc(hidden)]
pub use chrono;

#[cfg(feature = "chrono")]
#[doc(hidden)]
#[macro_export]
macro_rules! chrono_check {
    ($code: block, $error: block) => {
        $code
    };
}

#[cfg(not(feature = "chrono"))]
#[doc(hidden)]
#[macro_export]
macro_rules! chrono_check {
    ($code: block, $error: block) => {
        $error
    };
}
