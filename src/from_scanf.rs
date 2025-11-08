use crate::advanced::{FormatOptions, MatchTree, Matcher};

mod impls {
    mod numeric;
    mod other;
}

#[allow(unused_imports)]
use std::str::FromStr; // for links in docs

/// A trait that allows you to use a custom regex for parsing a type.
///
/// There are three options to implement this trait:
/// - [`#[derive(FromScanf)]` (simple, readable, fool proof (mostly))](#option-1-deriving)
/// - [manually implement `FromScanfSimple` (flexible, but requires more code)](#option-2-manually-implement-fromscanfsimple)
/// - [manually implement `FromScanf` (maximum flexibility, maximum complexity)](#option-3-manually-implement-fromscanf)
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
/// As you can see, the derive macro automatically generates the necessary code to parse the type from the format
/// string. It is aware of the types of the fields, so it can generate the correct regex and parser
/// implementation.
///
/// A detailed description of the syntax and options can be found [here](derive.FromScanf.html)
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
///     const REGEX: &'static str = r"[-+]?\d+/\d+"; // (sign) digits '/' digits
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
/// This option gives more control over the parsing process, but requires more code and manually writing the
/// regex/parsing.
///
/// Note that this option is especially useful for types that already implement [`FromStr`], since the parsing
/// logic can be reused. For example, the above implementation could be simplified to:
///
/// ```
/// # #[derive(Debug, PartialEq)] // additional traits for assert_eq below. Not required for sscanf and thus hidden in the example.
/// struct Fraction {
///     numerator: isize,
///     denominator: usize,
/// }
///
/// // existing FromStr implementation for Fraction
/// impl std::str::FromStr for Fraction {
///     type Err = &'static str; // simplified error type
///     fn from_str(input: &str) -> Result<Self, Self::Err> {
///         let (numerator, denominator) = input.split_once('/').ok_or("Missing '/'")?;
///         Ok(Self {
///             numerator: numerator.parse().map_err(|_| "Invalid numerator")?,
///             denominator: denominator.parse().map_err(|_| "Invalid denominator")?,
///         })
///     }
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
///     fn get_matcher(format: &FormatOptions) -> Matcher {
///         // matches <isize> '/' <usize>
///         Matcher::Seq(vec![
///             <isize as FromScanf>::get_matcher(format).into(),
///             MatchPart::literal("/"),
///             <usize as FromScanf>::get_matcher(format).into(),
///         ])
///     }
///
///     fn from_match_tree(matches: MatchTree<'_, '_>, format: &FormatOptions) -> Option<Self> {
///         let matches = matches.as_seq(); // our matcher is a sequence, so we can convert to that
///         Some(Self {
///             numerator: matches.parse_field("numerator", 0, format)?,
///             denominator: matches.parse_field("denominator", 2, format)?, // index 1 is the literal '/', so we skip it
///         })
///     }
/// }
///
/// let parsed = sscanf::sscanf!("-10/3", "{Fraction}").unwrap();
/// assert_eq!(parsed, Fraction { numerator: -10, denominator: 3 });
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
///     fn from_match_tree(matches: MatchTree<'_, 'input>, _: &FormatOptions) -> Option<Self> {
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
    /// # use sscanf::advanced::{Matcher, MatchTree, FormatOptions};
    /// # struct MyType { first_field: u8, second_field: u8 }
    /// impl sscanf::FromScanf<'_> for MyType {
    ///     fn get_matcher(_: &FormatOptions) -> Matcher {
    ///         Matcher::from_regex(r"your-(capturing)-(regex)-here").unwrap()
    ///     }
    ///
    ///     fn from_match_tree(matches: MatchTree<'_, '_>, _: &FormatOptions) -> Option<Self> {
    ///         let matches = matches.as_raw(); // our matcher is a raw regex, so we can convert to that
    ///         Some(Self {
    ///             first_field: matches.get(0).unwrap().parse().ok()?,
    ///             second_field: matches.get(1).unwrap().parse().ok()?,
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
    /// For types implementing [`FromStr`], this can just be `input.parse().ok()`:
    /// ```
    /// # struct MyType;
    /// # impl std::str::FromStr for MyType { type Err = (); fn from_str(_: &str) -> Result<Self, Self::Err> { Ok(MyType) } }
    /// impl sscanf::FromScanfSimple<'_> for MyType {
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
    fn from_match_tree(matches: MatchTree<'_, 'input>, _: &FormatOptions) -> Option<Self> {
        Self::from_match(matches.text())
    }
}
