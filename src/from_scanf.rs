use std::error::Error;
use std::str::FromStr;

use crate::errors::FromStrFailedError;

/// A trait that allows you to use a custom regex for parsing a type.
///
/// There are three options to implement this trait:
/// - `#[derive(FromScanf)]` (recommended)
/// - implement [`std::str::FromStr`] and relying on the [blanket implementation](#impl-FromScanf<%27t>)
/// - manual implementation (highly discouraged)
///
/// The second and third options also require you to implement [`RegexRepresentation`](crate::RegexRepresentation),
/// unless you **always** use a custom regex `{:/.../}`, but that tends to make the code less readable.
///
/// ## Option 1: Deriving
/// ```
/// #[derive(sscanf::FromScanf)]
/// #[sscanf(format = "#{r:r16}{g:r16}{b:r16}")] // matches '#' followed by 3 hexadecimal u8s
/// struct Color {                               // note the use of :r16 over :x to avoid prefixes
///     r: u8,
///     g: u8,
///     b: u8,
/// }
///
/// let input = "color: #ff12cc";
/// let parsed = sscanf::sscanf!(input, "color: {Color}").unwrap();
/// assert_eq!(parsed.r, 0xff);
/// assert_eq!(parsed.g, 0x12);
/// assert_eq!(parsed.b, 0xcc);
/// ```
///
/// A detailed description of the syntax and options can be found [here](derive.FromScanf.html)
///
/// ## Option 2: Implementing [`FromStr`]
/// ```
/// struct Color {
///     r: u8,
///     g: u8,
///     b: u8,
/// }
///
/// impl sscanf::RegexRepresentation for Color {
///     // matches '#' followed by 6 hexadecimal digits
///     const REGEX: &'static str = r"#[0-9a-fA-F]{6}";
/// }
///
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
///
/// let input = "color: #ff12cc";
/// let parsed = sscanf::sscanf!(input, "color: {Color}").unwrap();
/// assert_eq!(parsed.r, 0xff); assert_eq!(parsed.g, 0x12); assert_eq!(parsed.b, 0xcc);
/// ```
/// This option gives a lot more control over the parsing process, but requires more code and
/// manual error handling.
///
/// ## Option 3: Manual implementation
/// This should only be done if absolutely necessary, since it requires upholding several
/// conditions that cannot be properly checked by `sscanf`.
/// ```
/// # #[derive(Debug, PartialEq)]
/// struct Color {
///     r: u8,
///     g: u8,
///     b: u8,
/// }
///
/// impl sscanf::RegexRepresentation for Color {
///     // matches '#' followed by 3 capture groups with 2 hexadecimal digits each
///     //
///     // Capture groups are normally not allowed in RegexRepresentation, because the default
///     // `FromStr` blanket implementation does not handle them. Since this is a manual
///     // implementation of `FromScanf`, we can handle them ourselves.
///     const REGEX: &'static str = r"#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})";
///     //                            # \_____r______/  \_____g______/  \_____b______/
/// }
///
/// impl sscanf::FromScanf<'_> for Color {
///     /// The Error type in case parsing fails. In this case it is set to never fail (Infallible),
///     /// since if the above regex matches, the parsing cannot fail.
///     type Err = std::convert::Infallible;
///     const NUM_CAPTURES: usize = 4; // 3 capture groups + the whole match
///     fn from_matches(src: &mut regex::SubCaptureMatches) -> Result<Self, Self::Err> {
///         let _ = src.next().unwrap().unwrap(); // skip the whole match
///         // note the double-unwrap, since SubCaptureMatches::next() returns an Option<Option<Match>>
///
///         // checking the prefix is not necessary here, since the regex already enforces it
///
///         let r_str = src.next().unwrap().unwrap().as_str(); // unwrap is ok because the regex only matches if all capture groups match
///         let r = u8::from_str_radix(r_str, 16).unwrap();
///         let g_str = src.next().unwrap().unwrap().as_str();
///         let g = u8::from_str_radix(g_str, 16).unwrap();
///         let b_str = src.next().unwrap().unwrap().as_str();
///         let b = u8::from_str_radix(b_str, 16).unwrap();
///         // note that every result can be unwrapped here:
///         // This is possible because this trait is only used on a match to the RegexRepresentation::REGEX,
///         // which guarantees that everything is in the correct format. This means that the matched
///         // text for each capture group is guaranteed to be a valid u8 in hexadecimal format.
///
///         Ok(Color { r, g, b })
///     }
/// }
///
/// let input = "color: #ff12cc";
/// let parsed = sscanf::sscanf!(input, "color: {Color}").unwrap();
/// assert_eq!(parsed.r, 0xff); assert_eq!(parsed.g, 0x12); assert_eq!(parsed.b, 0xcc);
/// ```
/// This option usually has a faster runtime than [`FromStr`], since it can have capture groups
/// match during the initial parsing instead of having to parse the string again in the
/// [`FromStr`] implementation.
///
/// The downside is that it requires manually upholding the [`NUM_CAPTURES`](FromScanf::NUM_CAPTURES)
/// contract, which cannot be checked at compile time. It is also mostly not checked at runtime,
/// since this would require overhead that is unnecessary in all intended cases. This means that
/// an error in one implementation might cause a panic in another implementation, which is
/// near-impossible to debug.
///
/// The contract is:
/// - `NUM_CAPTURES` **IS EQUAL TO**
/// - the number of consumed elements from the iterator passed to [`from_matches`](FromScanf::from_matches) **IS EQUAL TO**
/// -  1 + the number of unescaped capture groups in [`RegexRepresentation`](crate::RegexRepresentation) (or `{:/.../}`).
/// The 1 is for the whole match, which is a capture group added by `sscanf`.
///
/// All of these are automatically enforced by the derive macro or the [`FromStr`] implementation,
/// which is why they should be preferred over this option.
///
/// #### Lifetime Parameter
/// The lifetime parameter of `FromScanf` is the borrow from the input string given to `sscanf`.
/// If your type borrows parts of that string, like `&str` does, you need to specify the lifetime
/// parameter and match it with the _second_ lifetime parameter of [`regex::SubCaptureMatches`]:
/// ```
/// struct Name<'a, 'b> {
///     first: &'a str,
///     last: &'b str,
/// }
///
/// impl<'a, 'b> sscanf::RegexRepresentation for Name<'a, 'b> {
///     const REGEX: &'static str = r"(\w+) (\w+)";
/// }
///
/// impl<'t> sscanf::FromScanf<'t> for Name<'t, 't> { // both parts are given the same input => same lifetime
///     type Err = std::convert::Infallible;
///     const NUM_CAPTURES: usize = 3;
///     fn from_matches(src: &mut regex::SubCaptureMatches<'_, 't>) -> Result<Self, Self::Err> {
///         let _ = src.next().unwrap().unwrap(); // skip the whole match
///         let first = src.next().unwrap().unwrap().as_str();
///         let last = src.next().unwrap().unwrap().as_str();
///         Ok(Self { first, last })
///     }
/// }
///
/// let input = String::from("John Doe");
/// let parsed = sscanf::sscanf!(input, "{Name}").unwrap();
/// assert_eq!(parsed.first, "John");
/// assert_eq!(parsed.last, "Doe");
/// ```
///
/// This allows custom borrows from the input string to avoid unnecessary allocations. The lifetime
/// of the returned value is that of the input string:
///
/// ```compile_fail
/// # #[derive(sscanf::FromScanf)]
/// # #[sscanf(format = "{} {}")]
/// struct Name<'a, 'b> {
///     first: &'a str,
///     last: &'b str,
/// }
/// // ...same impl setup as above...
///
/// let parsed;
/// {
///     let input = String::from("John Doe"); // owned string
///     parsed = sscanf::sscanf!(input, "{Name}").unwrap();
///     // input is dropped here
/// }
/// println!("{} {}", parsed.first, parsed.last); // use after drop
/// ```
pub trait FromScanf<'t>
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
    fn from_matches(src: &mut regex::SubCaptureMatches<'_, 't>) -> Result<Self, Self::Err>;

    /// Convenience shortcut for directly using this trait.
    ///
    /// If you have a string containing just the formatted version of the implementing type without
    /// any text around it, it would normally still be necessary to call
    /// ```ignore
    /// sscanf::sscanf!(input, "{<type>}")
    /// ```
    /// in order to use the [`FromScanf`] implementation.
    ///
    /// This method allows you to call
    /// ```ignore
    /// <type>::from_str(input)
    /// ```
    /// instead.
    ///
    /// On types that were auto-implemented based on their [`FromStr`] implementation, this method
    /// is functionally identical to [`FromStr::from_str`].
    ///
    /// Note that the returned [`Error`](crate::errors::Error) is the same as the one returned by
    /// [`sscanf!`](crate::sscanf), potentially wrapping a [`FromScanf::Err`].
    fn from_str(src: &'t str) -> Result<Self, crate::errors::Error>
    where
        Self: crate::RegexRepresentation,
    {
        let regex = format!("^{}$", Self::REGEX);
        let regex = crate::regex::Regex::new(&regex).unwrap_or_else(|err| {
            panic!(
                "sscanf: Type {} has invalid RegexRepresentation `{}`: {}",
                std::any::type_name::<Self>(),
                Self::REGEX,
                err
            )
        });

        regex
            .captures(src)
            .ok_or_else(|| crate::errors::Error::MatchFailed)
            .and_then(|cap| {
                let mut src = cap.iter();

                Self::from_matches(&mut src)
                    .map_err(|e| crate::errors::Error::ParsingFailed(Box::new(e)))
            })
    }
}

impl<'t, T> FromScanf<'t> for T
where
    T: FromStr + 'static,
    <T as FromStr>::Err: Error + 'static,
{
    type Err = FromStrFailedError<T>;
    const NUM_CAPTURES: usize = 1;
    fn from_matches(src: &mut regex::SubCaptureMatches<'_, 't>) -> Result<Self, Self::Err> {
        src.next()
            .expect(crate::errors::EXPECT_NEXT_HINT)
            .expect(crate::errors::EXPECT_CAPTURE_HINT)
            .as_str()
            .parse()
            .map_err(Self::Err::new)
    }
    fn from_str(src: &'t str) -> Result<Self, crate::errors::Error> {
        src.parse()
            .map_err(Self::Err::new)
            .map_err(|e| crate::errors::Error::ParsingFailed(Box::new(e)))
    }
}

#[doc(hidden)]
pub use FromScanf as FromSscanf;
