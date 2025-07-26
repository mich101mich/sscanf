mod impls;

/// A trait that allows you to use a custom regex for parsing a type.
///
/// There are two options to implement this trait:
/// - `#[derive(FromScanf)]` (simple, readable, fool proof (mostly))
/// - manual implementation (flexible, but requires more code)
///
/// ## Option 1: Deriving
/// ```
/// #[derive(sscanf::FromScanf)]
/// #[sscanf(format = "#{r:r16}{g:r16}{b:r16}")] // matches '#' followed by 3 hexadecimal u8s
/// struct Color {                               // note the use of :r16 over :x to avoid `0x` prefixes
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
/// ## Option 2: Manual Implementation
/// ```
/// struct Color {
///     r: u8,
///     g: u8,
///     b: u8,
/// }
///
/// impl sscanf::FromScanf<'_> for Color {
///     // matches '#' followed by 3 capture groups of 2 hexadecimal digits each
///     const REGEX: &'static str = r"#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})";
///
///     fn from_matches(_full_match: &str, sub_matches: &[Option<&str>]) -> Option<Self> {
///         // The full match is the entire matched string including the '#', but we don't need it
///         // here since we created capture groups in our regex.
///         // Those groups can be accessed through sub_matches.
///         Some(Self {
///             r: u8::from_str_radix(sub_matches[0].unwrap(), 16).unwrap(),
///             g: u8::from_str_radix(sub_matches[1].unwrap(), 16).unwrap(),
///             b: u8::from_str_radix(sub_matches[2].unwrap(), 16).unwrap(),
///         })
///         // note the use of `unwrap()` here, since the input to this function is guaranteed to
///         // match the regex and so we know that the contents are valid.
///     }
/// }
///
/// let input = "color: #ff12cc";
/// let parsed = sscanf::sscanf!(input, "color: {Color}").unwrap();
/// assert_eq!(parsed.r, 0xff); assert_eq!(parsed.g, 0x12); assert_eq!(parsed.b, 0xcc);
/// ```
/// This option gives a lot more control over the parsing process, but requires more code and
/// manually writing the regex/parsing.
///
/// #### Lifetime Parameter
/// The lifetime parameter of `FromScanf` is the borrow from the input string given to `sscanf`.
/// If your type borrows parts of that string, like `&str` does, you need to specify the lifetime
/// parameter and match it with the `'input` parameter:
/// ```
/// struct Name<'a, 'b> {
///     first: &'a str,
///     last: &'b str,
/// }
///
/// impl<'input> FromScanf<'input> for Name<'input, 'input> {
///     // both parts are given the same input => same lifetime
///
///     const REGEX: &'static str = r"(\w+) (\w+)";
///
///     fn from_matches(_full_match: &'input str, sub_matches: &[Option<&'input str>]) -> Option<Self> {
///         Some(Self {
///             first: sub_matches[0].unwrap(),
///             last: sub_matches[1].unwrap(),
///         })
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
/// # #[sscanf("{first} {last}")]
/// struct Name<'a, 'b> {
///     first: &'a str,
///     last: &'b str,
/// }
/// // ...same impl setup as above...
///
/// let parsed;
/// {
///     let input = String::from("John Doe"); // locally owned string
///     parsed = sscanf::sscanf!(input, "{Name}").unwrap();
///     // input is dropped here
/// }
/// println!("{} {}", parsed.first, parsed.last); // use after drop
/// ```
///
/// Note that lifetimes are automatically handled when deriving, though this is based on checking through the
/// provided types and their lifetimes, so it may not always be correct.
/// ```
/// #[derive(sscanf::FromScanf)]
/// #[sscanf("{first} {last}")]
/// struct Name<'a, 'b> {
///     first: &'a str,
///     last: &'b str,
/// }
///
/// let input = String::from("John Doe");
/// let parsed = sscanf::sscanf!(input, "{Name}").unwrap();
/// assert_eq!(parsed.first, "John");
/// assert_eq!(parsed.last, "Doe");
/// ```
///
#[diagnostic::on_unimplemented(
    message = "the type `{Self}` does not implement `FromScanf`",
    label = "`{Self}` does not implement `FromScanf`",
    note = "derive or implement `FromScanf` for `{Self}` to use it with `sscanf!`"
)]
pub trait FromScanf<'input>
where
    Self: Sized,
{
    /// A regular Expression that exactly matches any String representation of the implementing Type
    const REGEX: &'static str;

    /// The implementation of the parsing.
    ///
    /// For types implementing [`FromStr`](std::str::FromStr), this can just be
    /// ```
    /// fn from_matches(input: &str, _: &[&str]) -> Option<Self> {
    ///     input.parse().ok()
    /// }
    /// ```
    fn from_matches(full_match: &'input str, sub_matches: &[Option<&'input str>]) -> Option<Self>;
}

#[doc(hidden)]
pub use FromScanf as FromSscanf;
