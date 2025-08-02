use crate::advanced::{FormatOptions, MatchTree, Matcher};

mod impls {
    mod numeric;
    mod other;
}

/// A trait that allows you to use a custom regex for parsing a type.
///
/// There are three options to implement this trait:
/// - `#[derive(FromScanf)]` (simple, readable, fool proof (mostly))
/// - manually implement [`FromScanfSimple`] (flexible, but requires a bit more code)
/// - manually implement [`FromScanf`] (maximum flexibility, maximum complexity)
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
/// ## Option 2: Manually Implement `FromScanfSimple`
/// ```
/// struct Color {
///     r: u8,
///     g: u8,
///     b: u8,
/// }
///
/// impl sscanf::FromScanfSimple<'_> for Color {
///     // matches '#' followed by 3 hexadecimal digits
///     const REGEX: &'static str = r"#[0-9a-fA-F]{6}";
///
///     fn from_match(input: &str) -> Option<Self> {
///         Some(Self {
///             r: u8::from_str_radix(&input[1..3], 16).unwrap(),
///             g: u8::from_str_radix(&input[3..5], 16).unwrap(),
///             b: u8::from_str_radix(&input[5..7], 16).unwrap(),
///         })
///         // note the use of hardcoded lengths and `unwrap()` here, since the input to this
///         // function is guaranteed to match the regex and so we know that the contents are valid
///     }
/// }
///
/// let input = "color: #ff12cc";
/// let parsed = sscanf::sscanf!(input, "color: {Color}").unwrap();
/// assert_eq!(parsed.r, 0xff); assert_eq!(parsed.g, 0x12); assert_eq!(parsed.b, 0xcc);
/// ```
/// This option gives more control over the parsing process, but requires more code and manually writing the
/// regex/parsing.
///
/// ## Option 3: Manually Implement `FromScanf`
/// ```
/// struct Color {
///     r: u8,
///     g: u8,
///     b: u8,
/// }
///
/// use sscanf::advanced::{FromScanf, Matcher, MatcherComponent, FormatOptions, MatchTree};
/// impl FromScanf<'_> for Color {
///     // matches '#' followed by 3 capture groups of 2 hexadecimal digits each
///     fn get_matcher(_: &FormatOptions) -> Matcher {
///         let hex_format = FormatOptions::builder().hex().build();
///         let hex_u8_matcher: MatcherComponent = u8::get_matcher(&hex_format).into();
///         Matcher::from_sequence(vec![
///             MatcherComponent::Literal(r"#"),
///             hex_u8_matcher.clone(),
///             hex_u8_matcher.clone(),
///             hex_u8_matcher,
///         ])
///         // alternatively, we could write
///         //     `Matcher::from_regex(r"#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})")`
///     }
///
///     fn from_match_tree(matches: MatchTree<'_, '_>, _: &FormatOptions) -> Option<Self> {
///         // We can access the `Matcher` instances in the get_matcher method to create through
///         // the `MatchTree` structure.
///         let hex_format = FormatOptions::builder().hex().build();
///         Some(Self {
///             r: matches.parse_at(0, &hex_format)?,
///             // or: r: u8::from_str_radix(matches.at(0).text(), 16).unwrap(),
///             g: matches.parse_at(1, &hex_format)?,
///             b: matches.parse_at(2, &hex_format)?,
///         })
///     }
/// }
///
/// let input = "color: #ff12cc";
/// let parsed = sscanf::sscanf!(input, "color: {Color}").unwrap();
/// assert_eq!(parsed.r, 0xff); assert_eq!(parsed.g, 0x12); assert_eq!(parsed.b, 0xcc);
/// ```
/// This option gives a lot of control over the matching and parsing process. It is also generally faster than the
/// [`FromScanfSimple`] option, since we can directly access the capture groups without having to parse the string
/// again. In return, a lot more code is required and it is far more complex to implement/maintain.
///
/// Hence why it is recommended to use the derive macro to abstract away the complexity of this option while still
/// getting the same performance benefits.
///
/// #### Lifetime Parameter
/// The lifetime parameter of `FromScanf` and `FromScanfSimple` is the borrow from the input string given to `sscanf`.
/// If your type borrows parts of that string, like `&str` does, you need to specify the lifetime
/// parameter and match it with the `'input` parameter:
/// ```
/// struct Name<'a, 'b> {
///     first: &'a str,
///     last: &'b str,
/// }
///
/// impl<'input> sscanf::FromScanfSimple<'input> for Name<'input, 'input> {
///     // both parts are given the same input => same lifetime
///
///     const REGEX: &'static str = r"\w+ \w+";
///
///     fn from_match(input: &str) -> Option<Self> {
///         let (first, last) = input.split_once(' ')?; // has lifetime 'input
///         Some(Self {
///             first,
///             last,
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
pub trait FromScanf<'input>: Sized {
    /// A regular expression that exactly matches any string representation of the implementing type
    ///
    /// TODO: give hints on how to create this regex
    fn get_matcher(format: &FormatOptions) -> Matcher;

    /// Callback to parse the input string from a match tree.
    ///
    /// ```
    /// # struct MyType { first_field: u8, second_field: u8 }
    /// impl sscanf::advanced::FromScanf<'_> for MyType {
    ///     fn get_matcher() -> sscanf::advanced::Matcher {
    ///         sscanf::advanced::Matcher::from_regex(r"your-(capturing)-(regex)-here")
    ///     }
    ///
    ///     fn from_match_tree(matches: sscanf::advanced::MatchTree<'_, '_>) -> Option<Self> {
    ///         Some(Self {
    ///             first_field: matches.parse_at(0)?,
    ///             second_field: matches.parse_at(1)?,
    ///             // ...
    ///         })
    ///     }
    /// }
    /// ```
    fn from_match_tree(matches: MatchTree<'_, 'input>, format: &FormatOptions) -> Option<Self>;
}

/// A simpler version of [`FromScanf`] for manual implementations.
#[diagnostic::on_unimplemented(
    message = "type `{Self}` can't be parsed by sscanf because it does not implement `FromScanf`",
    label = "can't be parsed by sscanf",
    note = "derive or implement `FromScanfSimple` for `{Self}` to use it with `sscanf!`",
    note = "see the documentation for details: <https://docs.rs/sscanf/latest/sscanf/trait.FromScanfSimple.html>"
)]
pub trait FromScanfSimple<'input>
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
    fn from_match(input: &'input str) -> Option<Self>;
}

impl<'input, T: FromScanfSimple<'input>> FromScanf<'input> for T {
    fn get_matcher(_: &FormatOptions) -> Matcher {
        Matcher::from_regex(T::REGEX)
    }

    fn from_match_tree(matches: MatchTree<'_, 'input>, _: &FormatOptions) -> Option<Self> {
        Self::from_match(matches.text())
    }
}
