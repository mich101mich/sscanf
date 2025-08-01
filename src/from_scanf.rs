use crate::MatchTree;

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
///     fn from_match(_: &str) -> Option<Self> { None }
///
///     fn from_match_tree(matches: sscanf::MatchTree<'_, '_>) -> Option<Self> {
///         // The matches.full is the entire matched string including the '#', but we don't need it
///         // here since we created capture groups in our regex.
///         // Those groups can be accessed through matches.inner.
///         Some(Self {
///             r: u8::from_str_radix(matches.at(0).text(), 16).unwrap(),
///             g: u8::from_str_radix(matches.at(1).text(), 16).unwrap(),
///             b: u8::from_str_radix(matches.at(2).text(), 16).unwrap(),
///         })
///         // note the use of `at()` and `unwrap()` here, since the input to this function is
///         // guaranteed to match the regex and so we know that the contents are valid.
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
/// impl<'input> sscanf::FromScanf<'input> for Name<'input, 'input> {
///     // both parts are given the same input => same lifetime
///
///     const REGEX: &'static str = r"(\w+) (\w+)";
///
///     fn from_match(_: &str) -> Option<Self> { None }
///
///     fn from_match_tree(matches: sscanf::MatchTree<'_, 'input>) -> Option<Self> {
///         Some(Self {
///             first: matches.at(0).text(), // has lifetime 'input
///             last: matches.at(1).text(),
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
    message = "type `{Self}` can't be parsed by sscanf because it does not implement `FromScanf`",
    label = "can't be parsed by sscanf",
    note = "derive or implement `FromScanf` for `{Self}` to use it with `sscanf!`",
    note = "see the `FromScanf` documentation for details: <https://docs.rs/sscanf/latest/sscanf/trait.FromScanf.html>"
)]
pub trait FromScanf<'input>
where
    Self: Sized,
{
    /// A regular expression that exactly matches any string representation of the implementing type
    ///
    /// TODO: give hints on how to create this regex
    const REGEX: &'static str;

    /// The implementation of the parsing.
    ///
    /// For types implementing [`FromStr`](std::str::FromStr), this can just be `input.parse().ok()`:
    /// ```
    /// # struct MyType;
    /// # impl std::str::FromStr for MyType { type Err = (); fn from_str(_: &str) -> Result<Self, Self::Err> { Ok(MyType) } }
    /// impl sscanf::FromScanf<'_> for MyType {
    ///     const REGEX: &'static str = // your regex here
    /// # "placeholder regex to make this compile";
    ///
    ///     fn from_match(input: &str) -> Option<Self> {
    ///         input.parse().ok()
    ///     }
    /// }
    /// ```
    ///
    /// If your type uses capture groups in the regex, override the [`from_match_tree`](Self::from_match_tree) method
    /// to access the submatches and have this method just return `None`.
    fn from_match(input: &'input str) -> Option<Self>;

    /// More advanced version of [`from_match`](Self::from_match) that allows access to submatches.
    ///
    /// By default, it is assumed that capture groups are not used, so this method just calls
    /// [`from_match`](Self::from_match) with the full match.
    ///
    /// If you do need to use capture groups, you can override this method to access the submatches:
    /// ```
    /// # struct MyType { first_field: u8, second_field: u8 }
    /// impl sscanf::FromScanf<'_> for MyType {
    ///     const REGEX: &'static str = // your regex here (with capture groups)
    /// # "(placeholder) (regex) to make this compile";
    ///
    ///     fn from_match(_: &str) -> Option<Self> {
    ///         None // This function won't ever be called if from_match_tree is overridden
    ///     }
    ///
    ///     fn from_match_tree(matches: sscanf::MatchTree<'_, '_>) -> Option<Self> {
    ///         Some(Self {
    ///             first_field: matches.parse_at(0)?,
    ///             second_field: matches.parse_at(1)?,
    ///             // ...
    ///         })
    ///     }
    /// }
    /// ```
    fn from_match_tree(matches: MatchTree<'_, 'input>) -> Option<Self> {
        Self::from_match(matches.text())
    }
}

#[doc(hidden)]
pub use FromScanf as FromSscanf;
