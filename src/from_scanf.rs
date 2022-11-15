use std::error::Error;
use std::str::FromStr;

use crate::FromStrFailedError;

/// A trait that allows you to use a custom regex for parsing a type.
///
/// **You should not implement this trait yourself.**
///
/// There are three options to implement this trait:
/// - `#[derive(FromScanf)]` (recommended)
/// - implement [`FromStr`] and use the [blanket implementation](#impl-FromScanf)
/// - manual implementation (highly discouraged)
///
/// The second and third options also require you to implement [`RegexRepresentation`](crate::RegexRepresentation),
/// unless you **always** use a custom regex `{:/.../}`, but this tends to make the code less readable.
///
/// ## Deriving
/// ```
/// use sscanf::{sscanf, FromScanf};
///
/// #[derive(FromScanf)]
/// # #[derive(Debug, PartialEq)]
/// #[sscanf(format = "#{r:r16}{g:r16}{b:r16}")] // matches '#' followed by 3 hexadecimal u8s
///                                              // note the use of :r16 over :x to avoid prefixes
/// struct Color {
///     r: u8,
///     g: u8,
///     b: u8,
/// }
///
/// let input = "color: #ff00cc";
/// let parsed = sscanf!(input, "color: {Color}").unwrap();
/// assert_eq!(parsed, Color { r: 0xff, g: 0x00, b: 0xcc });
/// ```
///
/// A detailed description of the syntax and options can be found [here](derive.FromScanf.html)
///
/// ## Implementing [`FromStr`]
/// ```
/// # #[derive(Debug, PartialEq)]
/// struct Color {
///     r: u8,
///     g: u8,
///     b: u8,
/// }
/// impl sscanf::RegexRepresentation for Color {
///     // matches '#' followed by 6 hexadecimal digits
///     const REGEX: &'static str = r"#[0-9a-fA-F]{6}";
/// }
/// #[derive(Debug, thiserror::Error)]
/// enum ColorParseError {
///     #[error("Invalid digit")]
///     InvalidHexDigit(#[from] std::num::ParseIntError),
///     #[error("Expected 6 hex digits, found {0}")]
///     InvalidLength(usize),
///     #[error("Color has to be prefixed with '#'")]
///     InvalidPrefix,
/// }
/// impl std::str::FromStr for Color {
///     type Err = ColorParseError;
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         let s = s.strip_prefix('#')
///                  .ok_or(ColorParseError::InvalidPrefix)?;
///         if s.len() != 6 {
///             return Err(ColorParseError::InvalidLength(s.len()));
///         }
///         let r = u8::from_str_radix(&s[0..2], 16)?;
///         let g = u8::from_str_radix(&s[2..4], 16)?;
///         let b = u8::from_str_radix(&s[4..6], 16)?;
///         Ok(Color { r, g, b })
///     }
/// }
/// let input = "color: #ff00cc";
/// let parsed = sscanf::sscanf!(input, "color: {Color}").unwrap();
/// assert_eq!(parsed, Color { r: 0xff, g: 0x00, b: 0xcc });
/// ```
/// This option gives a lot more control over the parsing process, but requires more code and
/// manual error handling.
///
/// ## Manual implementation
/// This should only be done if absolutely necessary, since it requires upholding several invariants
/// that cannot be checked at compile time.
/// ```
/// use sscanf::{FromScanf, RegexRepresentation};
///
/// # #[derive(Debug, PartialEq)]
/// struct Color {
///     r: u8,
///     g: u8,
///     b: u8,
/// }
///
/// impl RegexRepresentation for Color {
///     // matches '#' followed by 3 capture groups with 2 hexadecimal digits each
///     //
///     // Capture groups are normally not allowed in RegexRepresentation, because the default
///     // `FromStr` blanket implementation does not handle them. Since this is a manual
///     // implementation of `FromScanf`, we can handle them ourselves.
///     const REGEX: &'static str = r"#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})";
///     //                            # \_____r______/  \_____g______/  \_____b______/
/// }
///
/// impl FromScanf for Color {
///     type Err = std::num::ParseIntError;
///     const NUM_CAPTURES: usize = 4; // 3 capture groups + the whole match
///     fn from_matches(src: &mut regex::SubCaptureMatches) -> Result<Self, Self::Err> {
///         let _ = src.next().unwrap().unwrap(); // skip the whole match
///         // note the double-unwrap, since SubCaptureMatches::next() returns an Option<Option<Match>>
///
///         // checking the prefix is not necessary here, since the regex already enforces it
///
///         let r_str = src.next().unwrap().unwrap().as_str(); // unwrap is ok because the regex only matches if all capture groups match
///         let r = u8::from_str_radix(r_str, 16)?;
///         let g_str = src.next().unwrap().unwrap().as_str();
///         let g = u8::from_str_radix(g_str, 16)?;
///         let b_str = src.next().unwrap().unwrap().as_str();
///         let b = u8::from_str_radix(b_str, 16)?;
///         // instead of using '?' it is also technically possible to simply unwrap and set the
///         // Err-type to '!', since the regex only allows valid hex u8s, meaning that this
///         // conversion cannot fail
///
///         Ok(Color { r, g, b })
///     }
/// }
/// ```
/// This option may be faster than [`FromStr`], since it can have capture groups
/// match during the initial parsing instead of having to parse the string again in the
/// [`FromStr`] implementation.
///
/// The downside is that it requires manually upholding the [`NUM_CAPTURES`](FromScanf::NUM_CAPTURES)
/// invariant, which cannot be checked at compile time. It is also mostly not checked at runtime,
/// since this would require a lot of overhead. This means that an error in one implementation
/// might cause a panic in another implementation, which is near-impossible to debug.
///
/// The invariant is:
/// - `NUM_CAPTURES` **IS EQUAL TO**
/// - the number of consumed elements from the iterator passed to [`from_matches`](FromScanf::from_matches) **IS EQUAL TO**
/// -  1 + the number of unescaped capture groups in [`RegexRepresentation`](crate::RegexRepresentation) (or `{:/.../}`).
/// The 1 is for the whole match, which is a capture group added by `sscanf`.
///
/// All of these are automatically enforced by the derive macro or the [`FromStr`] implementation.
pub trait FromScanf
where
    Self: Sized,
{
    /// Error type
    type Err: Error + 'static;
    /// Number of captures taken by this regex.
    ///
    /// **HAS** to match the number of unescaped capture groups in the [`RegexRepresentation`](crate::RegexRepresentation)
    /// +1 for the whole match.
    const NUM_CAPTURES: usize;
    /// The implementation of the parsing.
    ///
    /// **HAS** to take **EXACTLY** `NUM_CAPTURES` elements from the iterator.
    fn from_matches(src: &mut regex::SubCaptureMatches) -> Result<Self, Self::Err>;
}

impl<T> FromScanf for T
where
    T: FromStr + 'static,
    <T as FromStr>::Err: Error + 'static,
{
    type Err = FromStrFailedError<T>;
    const NUM_CAPTURES: usize = 1;
    fn from_matches(src: &mut regex::SubCaptureMatches) -> Result<Self, Self::Err> {
        src.next()
            .expect(crate::EXPECT_NEXT_HINT)
            .expect(crate::EXPECT_CAPTURE_HINT)
            .as_str()
            .parse()
            .map_err(FromStrFailedError::new)
    }
}
