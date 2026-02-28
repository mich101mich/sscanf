use crate::advanced::{FormatOptions, Match, Matcher};

mod impls {
    mod numeric;
    mod other;
}

#[expect(unused_imports, reason = "for links in docs")]
use std::str::FromStr;

/// A trait for parsing a type with `sscanf`.
///
/// There are three ways to implement this trait:
/// - [`#[derive(FromScanf)]`](derive.FromScanf.html) (simple, readable, foolproof) - see [Option 1](#option-1-deriving)
/// - Manually implement [`FromScanfSimple`] (more flexible, more code) - see [Option 2](#option-2-manually-implement-fromscanfsimple)
/// - Manually implement [`FromScanf`] (maximum flexibility and complexity) - see [Option 3](#option-3-manually-implement-fromscanf)
///
/// ## Option 1: Deriving
/// ```
/// # #[derive(Debug, PartialEq)] // additional traits for assert_eq below. Not required for sscanf and thus hidden in the example.
/// #[derive(sscanf::FromScanf)] // The derive macro
/// #[sscanf(format = "{numerator}/{denominator}")] // Format string for the type, using the field names.
/// struct Fraction {
///     numerator: isize,
///     denominator: usize,
/// }
///
/// let parsed = sscanf::sscanf!("-10/3", "{Fraction}").unwrap();
/// assert_eq!(parsed, Fraction { numerator: -10, denominator: 3 });
/// ```
///
/// The derive macro generates the code to parse the type from the format string. It knows the field types and can
/// generate the correct matcher and parser implementation.
///
/// A detailed description of the syntax and options is available in [the derive documentation](derive.FromScanf.html).
///
/// ## Option 2: Manually Implement [`FromScanfSimple`]
/// ```
/// # #[derive(Debug, PartialEq)] // additional traits for assert_eq below. Not required for sscanf and thus hidden in the example.
/// struct Fraction {
///     numerator: isize,
///     denominator: usize,
/// }
///
/// impl sscanf::FromScanfSimple<'_> for Fraction {
///     const REGEX: &'static str = r"[-+]?[0-9]+/[0-9]+"; // (sign) digits '/' digits
///
///     fn from_match(input: &str) -> Option<Self> {
///         let (numerator, denominator) = input.split_once('/').unwrap(); // unwrap is safe here, since the regex guarantees the presence of '/'
///         Some(Self {
///             numerator: numerator.parse().ok()?,
///             denominator: denominator.parse().ok()?,
///         })
///     }
/// }
///
/// let parsed = sscanf::sscanf!("-10/3", "{Fraction}").unwrap();
/// assert_eq!(parsed, Fraction { numerator: -10, denominator: 3 });
/// ```
/// This option gives more control over parsing but requires more code and a regex.
///
/// This option is especially useful for types that already implement [`FromStr`], since the parsing logic can be
/// reused. For example, the above implementation could be simplified to:
///
/// ```
/// # #[derive(Debug, PartialEq)] // additional traits for assert_eq below. Not required for sscanf and thus hidden in the example.
/// struct Fraction {
///     numerator: isize,
///     denominator: usize,
/// }
///
/// impl std::str::FromStr for Fraction {
///     // existing FromStr implementation for Fraction
/// #   type Err = &'static str; // simplified error type
/// #   fn from_str(input: &str) -> Result<Self, Self::Err> {
/// #       let (numerator, denominator) = input.split_once('/').ok_or("Missing '/'")?;
/// #       Ok(Self {
/// #           numerator: numerator.parse().map_err(|_| "Invalid numerator")?,
/// #           denominator: denominator.parse().map_err(|_| "Invalid denominator")?,
/// #       })
/// #   }
/// }
///
/// impl sscanf::FromScanfSimple<'_> for Fraction {
///     const REGEX: &'static str = r"[-+]?\d+/\d+"; // (sign) digits '/' digits
///
///     fn from_match(input: &str) -> Option<Self> {
///         input.parse().ok() // reuse FromStr implementation
///     }
/// }
///
/// let parsed = sscanf::sscanf!("-10/3", "{Fraction}").unwrap();
/// assert_eq!(parsed, Fraction { numerator: -10, denominator: 3 });
/// ```
///
/// ## Option 3: Manually Implement [`FromScanf`]
/// ```
/// # use sscanf::FromScanf;
/// # #[derive(Debug, PartialEq)] // additional traits for assert_eq below. Not required for sscanf and thus hidden in the example.
/// struct Fraction {
///     numerator: isize,
///     denominator: usize,
/// }
///
/// use sscanf::advanced::*; // for Matcher etc.
/// impl FromScanf<'_> for Fraction {
///     fn get_matcher(options: &FormatOptions) -> Matcher {
///         // matches <isize> '/' <usize>
///         Matcher::Seq(vec![
///             <isize as FromScanf>::get_matcher(options).into(),
///             MatchPart::literal("/"),
///             <usize as FromScanf>::get_matcher(options).into(),
///         ])
///     }
///
///     fn from_match(matches: Match<'_, '_>, options: &FormatOptions) -> Option<Self> {
///         let matches = matches.as_seq(); // our matcher is a sequence, so we can convert to that
///         Some(Self {
///             numerator: matches.parse_field("numerator", 0, options)?,
///             denominator: matches.parse_field("denominator", 2, options)?, // index 1 is the literal '/', so we skip it
///         })
///     }
/// }
///
/// let parsed = sscanf::sscanf!("-10/3", "{Fraction}").unwrap();
/// assert_eq!(parsed, Fraction { numerator: -10, denominator: 3 });
/// ```
/// This option offers fine-grained control over matching and parsing. It is generally faster than
/// [`FromScanfSimple`], since you can access match results directly without reparsing the string. In return, it
/// requires more code and is more complex to implement and maintain.
///
/// Therefore, using the derive macro is recommended to hide this complexity while keeping the same performance.
///
/// #### Lifetime Parameter
/// The lifetime parameter of `FromScanf` and `FromScanfSimple` represents the borrow from the input string given to
/// `sscanf`. If your type borrows from that string (like `&str`), specify the lifetime and match it with the
/// `'input` parameter:
/// ```
/// struct Name<'a, 'b> {
///     first: &'a str,
///     last: &'b str,
/// }
///
/// use sscanf::advanced::*; // for Matcher etc.
/// impl<'input> sscanf::FromScanf<'input> for Name<'input, 'input> {
///     // both parts are given the same input => same lifetime
///
///     fn get_matcher(_: &FormatOptions) -> Matcher {
///         Matcher::Seq(vec![
///             Matcher::from_regex(r"\S+").unwrap().into(), // first name: non-whitespace characters
///             MatchPart::literal(" "),
///             Matcher::from_regex(r"\S+").unwrap().into(), // last name: non-whitespace characters
///         ])
///     }
///
///     fn from_match(matches: Match<'_, 'input>, _: &FormatOptions) -> Option<Self> {
///         let matches = matches.as_seq();
///         Some(Self {
///             first: matches.at(0).text(),
///             last: matches.at(2).text(), // index 1 is the space
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
/// This enables borrowing from the input string to avoid allocations. The returned value's lifetime is that of the
/// input string:
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
/// Deriving handles lifetimes automatically by inspecting provided types, though this may not always be perfect.
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
    /// Create a matcher to find and capture the string representation of the implementing type.
    ///
    /// See the documentation of [`Matcher`] for details on how to create matchers.
    ///
    /// The `options` parameter contains customizations from the format string, like `{:x}` for hexadecimal number
    /// parsing. If you want numbers within your type to be overridden by these options, you need to pass them
    /// down to the matchers of the fields. Otherwise, you can use, ignore, customize, or override this parameter as
    /// you see fit.
    fn get_matcher(options: &FormatOptions) -> Matcher;

    /// Callback to parse the input string from a match tree.
    ///
    /// ```
    /// # use sscanf::advanced::{Matcher, Match, FormatOptions};
    /// # struct MyType { first_field: u8, second_field: u8 }
    /// impl sscanf::FromScanf<'_> for MyType {
    ///     fn get_matcher(_: &FormatOptions) -> Matcher {
    ///         Matcher::from_regex(r"your-(capturing)-(regex)-here").unwrap()
    ///     }
    ///
    ///     fn from_match(matches: Match<'_, '_>, _: &FormatOptions) -> Option<Self> {
    ///         let matches = matches.as_regex_matches(); // our matcher used from_regex, so we can convert to that
    ///         Some(Self {
    ///             first_field: matches[0].unwrap().parse().ok()?,
    ///             second_field: matches[1].unwrap().parse().ok()?,
    ///             // ...
    ///         })
    ///     }
    /// }
    /// ```
    fn from_match(matches: Match<'_, 'input>, options: &FormatOptions) -> Option<Self>;
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
    /// A regular expression that matches any string representation of the implementing type.
    ///
    /// The parts of the input string that are matched by this regex will be passed to the
    /// [`from_match`](FromScanfSimple::from_match) function for parsing, so the main requirement for this regex
    /// is that it matches exactly the characters that are relevant for parsing the type.\
    /// For example, for an integer type, the regex should match digits and optional signs, but nothing extra.
    ///
    /// The regex doesn't have to be a strict 1:1 match for all valid inputs, but it should be a best-effort match.\
    /// Take for example number types. `i32` can represent numbers from `-2_147_483_648` to `2_147_483_647`, but a
    /// regex that matches all of these values would be extremely complex. Instead, a simpler regex
    /// that matches the correct number of digits is used. This means that inputs from `-9_999_999_999` to
    /// `9_999_999_999` would match the regex, but fail during parsing.\
    /// This is acceptable: the regex is still a best-effort match for valid inputs and preserves the correct number
    /// of digits. If it merely matched "any number of digits", parsing consecutive hex numbers without separators
    /// (which is supported) could fail because the first number would greedily match all digits before failing later.
    ///
    /// What "best effort" means depends on the type being implemented.
    const REGEX: &'static str;

    /// Parsing implementation.
    ///
    /// For types implementing [`FromStr`], this can just be `input.parse().ok()`:
    /// ```
    /// # struct MyFromStrType;
    /// # impl std::str::FromStr for MyFromStrType { type Err = (); fn from_str(_: &str) -> Result<Self, Self::Err> { Ok(MyFromStrType) } }
    /// impl sscanf::FromScanfSimple<'_> for MyFromStrType {
    ///     const REGEX: &'static str = // your regex here
    /// # "placeholder regex to make this compile";
    ///
    ///     fn from_match(input: &str) -> Option<Self> {
    ///         input.parse().ok()
    ///     }
    /// }
    /// ```
    ///
    /// # Guide to `panic!` vs `return None`
    ///
    /// The example converts the `Result` from `FromStr::from_str` to an `Option` with `ok()`, which returns `None` for
    /// errors. This is the recommended way to handle parsing errors here when the regex is not a strict 1:1 match for
    /// all valid inputs.
    ///
    /// The [example in the `FromScanfSimple` docs](trait.FromScanf.html#option-2-manually-implement-fromscanfsimple) used
    /// `unwrap()` to assert the presence of `/`. This is acceptable there, since the regex guarantees it.
    ///
    /// This is the rough guideline:
    /// - The input to this function is **guaranteed** to match [`REGEX`](FromScanfSimple::REGEX). Any violation is a
    ///   programming error in the calling code and should `panic!()`.
    ///   - This also includes mistakes in the regex itself, since it is a compile-time constant.
    /// - If the regex matched something the parser can't handle, return `None`.
    fn from_match(input: &'input str) -> Option<Self>;
}

#[diagnostic::do_not_recommend]
impl<'input, T: FromScanfSimple<'input>> FromScanf<'input> for T {
    fn get_matcher(_: &FormatOptions) -> Matcher {
        Matcher::from_regex(T::REGEX).unwrap_or_else(|err| {
            panic!(
                "sscanf: Invalid REGEX on FromScanfSimple of type {}: {err}",
                std::any::type_name::<T>()
            );
        })
    }

    #[track_caller]
    fn from_match(matches: Match<'_, 'input>, _: &FormatOptions) -> Option<Self> {
        Self::from_match(matches.text())
    }
}
