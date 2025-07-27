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
///     fn from_match_tree(matches: &sscanf::MatchTree<'_>) -> Option<Self> {
///         // The matches.full is the entire matched string including the '#', but we don't need it
///         // here since we created capture groups in our regex.
///         // Those groups can be accessed through matches.inner.
///         Some(Self {
///             r: u8::from_str_radix(matches.at(0).full, 16).unwrap(),
///             g: u8::from_str_radix(matches.at(1).full, 16).unwrap(),
///             b: u8::from_str_radix(matches.at(2).full, 16).unwrap(),
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
///     fn from_match_tree(matches: &sscanf::MatchTree<'input>) -> Option<Self> {
///         Some(Self {
///             first: matches.at(0).full, // has lifetime 'input
///             last: matches.at(1).full,
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
    /// A regular expression that exactly matches any string representation of the implementing type
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
    ///     fn from_match_tree(matches: &sscanf::MatchTree<'_>) -> Option<Self> {
    ///         Some(Self {
    ///             first_field: matches.at(0).parse()?,
    ///             second_field: matches.at(1).parse()?,
    ///             // ...
    ///         })
    ///     }
    /// }
    /// ```
    fn from_match_tree(matches: &'_ MatchTree<'input>) -> Option<Self> {
        Self::from_match(matches.full)
    }
}

/// Representation of the match of a capture group in a regex, arranged in a tree structure.
///
/// This type is the parameter to the [`FromScanf::from_match_tree`] method.
///
/// The `full` field contains the entire matched string, while `inner` contains
/// the matches of any inner capture groups within this capture group.
/// ```
/// # use sscanf::MatchTree;
/// # struct MyType;
/// impl sscanf::FromScanf<'_> for MyType {
///     const REGEX: &'static str = "a(b)c(x)?d(ef(ghi)j(k))lm";
///
///     fn from_match(_: &str) -> Option<Self> { None }
///
///     fn from_match_tree(matches: &MatchTree<'_>) -> Option<Self> {
///         // This is what the matches would look like:
///         assert_eq!(*matches, MatchTree {
///             full: "abcdefghijklm",
///             inner: vec![
///                 Some(MatchTree { // the "(b)" group
///                     full: "b",
///                     inner: vec![] // no more capture groups within this group
///                 }),
///                 None, // the "(x)?" group (did not match)
///                 Some(MatchTree { // the "(ef(ghi)j(k))" group
///                     full: "efghijk",
///                     inner: vec![
///                         Some(MatchTree { // the "(ghi)" group
///                             full: "ghi",
///                             inner: vec![]
///                         }),
///                         Some(MatchTree { // the "(k)" group
///                             full: "k",
///                             inner: vec![]
///                         })
///                     ]
///                 })
///             ]
///         });
///         // ... do something with the matches ...
///         # Some(MyType)
///     }
/// }
/// sscanf::sscanf!("abcdefghijklm", "{MyType}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchTree<'input> {
    /// The full match of this capture group
    pub full: &'input str,
    /// The matches of any inner capture groups within this capture group.
    ///
    /// Note that they are wrapped in `Option`, because not all capture groups are guaranteed to participate in a
    /// match. For example, if a capture group is optional `()?` or in an alternation `()|()`, then it may not match
    /// anything, resulting in `None` for that group.
    /// ```
    /// # #[derive(Debug, PartialEq, Eq)]
    /// enum MyType<'a> {
    ///     Digits(&'a str),
    ///     Letters(&'a str),
    /// }
    /// impl<'input> sscanf::FromScanf<'input> for MyType<'input> {
    ///     // matches either digits or letters, but not both
    ///     const REGEX: &'static str = r"(\d+)|([a-zA-Z]+)";
    ///
    ///     fn from_match(_: &str) -> Option<Self> { None }
    ///
    ///     fn from_match_tree(matches: &sscanf::MatchTree<'input>) -> Option<Self> {
    ///         if let Some(digits) = matches.get(0) {
    ///             assert!(matches.get(1).is_none()); // only one of the capture groups matches
    ///             Some(Self::Digits(digits.full))
    ///         } else {
    ///             // exactly one of the capture groups will match
    ///             let letters = matches.at(1);
    ///             Some(Self::Letters(letters.full))
    ///         }
    ///     }
    /// }
    ///
    /// let digits = sscanf::sscanf!("123", "{MyType}").unwrap();
    /// assert_eq!(digits, MyType::Digits("123"));
    ///
    /// let letters = sscanf::sscanf!("abc", "{MyType}").unwrap();
    /// assert_eq!(letters, MyType::Letters("abc"));
    /// ```
    ///
    /// Side note: This is the mechanism used by the derive macro when used on an enum. If the derive macro does not
    /// work for your enum, consider implementing this trait in this exact way, using alternations in the regex for the
    /// enum variants, each wrapped in a capture group to check which variant matched: `(...)|(...)|(...)`.
    pub inner: Vec<Option<MatchTree<'input>>>,
}

impl<'input> MatchTree<'input> {
    /// Convenience method to call [`FromScanf::from_match_tree`] with this match tree.
    ///
    /// The type `T` must implement the `FromScanf` trait, and this object must have been created from a match to
    /// `T::REGEX`.
    pub fn parse<T: FromScanf<'input>>(&self) -> Option<T> {
        T::from_match_tree(self)
    }

    /// Returns the inner match at the given index, if it participated in the match.
    ///
    /// ## Panics
    /// Panics if the index is out of bounds.
    #[track_caller]
    pub fn get(&self, index: usize) -> Option<&MatchTree<'input>> {
        let child = self.inner.get(index);
        let child = child
            .as_ref()
            .expect("sscanf: index out of bounds in MatchTree. Does the regex contain the correct number of capture groups?");
        child.as_ref()
    }

    /// Returns the inner match at the given index, asserting that it exists.
    ///
    /// ## Panics
    /// Panics if the index is out of bounds or if the inner match at that index is `None`.
    #[track_caller]
    pub fn at(&self, index: usize) -> &MatchTree<'input> {
        let child = self.inner.get(index);
        let child = child
            .as_ref()
            .expect("sscanf: index out of bounds in MatchTree. Does the regex contain the correct number of capture groups?");
        let Some(ret) = child else {
            // using let else because we need to format the error message but we don't want to create a non-track_caller function
            // by using unwrap_or_else
            panic!(
                "sscanf: inner match at index {index} is None. Does the regex contain the correct number of capture groups?"
            );
        };
        ret
    }
}

#[doc(hidden)]
pub use FromScanf as FromSscanf;
