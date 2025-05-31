//! Module for parsing custom types with sscanf

use std::str::FromStr;

pub mod format_options;
mod regex_segment;
mod sub_type;
mod impls {
    use super::*;
    mod numeric;
    mod other;
}

pub(crate) use format_options::*;
pub use regex_segment::*;
pub use sub_type::*;

/// The core trait for parsing custom types with sscanf. Can be derived with the `FromScanf` derive macro.
///
/// See the [derive macro documentation](crate::macros::FromScanf) for more information on how to use it.
///
/// <br>
/// <div class="warning">
///     This trait is mostly meant as the backend for the derive macro. Only implement manually if absolutely necessary.
/// </div>
/// <br>
///
/// If the derive macro does not work for your type, check if the simpler [`FromScanfSimple`] trait fits your use case.
///
/// Only if neither of those works, should you even consider implementing this trait manually. The reason for this is
/// that this trait is meant to give maximum flexibility, which also means that it is more complex to implement
/// and requires a decent understanding of the inner workings of the crate.
///
/// Here is an example of the three ways to implement this trait:
///
/// ### Using the derive macro
/// ```
/// # #[derive(Debug, PartialEq)] // additional traits for assert_eq below. Not required for the macro and thus hidden in the example.
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
/// ### Using the [`FromScanfSimple`] trait
/// ```
/// #[derive(Debug, PartialEq)]
/// struct Fraction {
///     numerator: isize,
///     denominator: usize,
/// }
///
/// // Trait for matching: FromScanfSimple
/// impl sscanf::FromScanfSimple for Fraction {
///     const REGEX: &'static str = r"[-+]?\d+/\d+"; // (sign) digits '/' digits
/// }
///
/// // Trait for parsing: FromStr
/// impl std::str::FromStr for Fraction {
///     type Err = String; // Simple error type for demonstration purposes.
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         let (numerator, denominator) = s
///             .split_once('/')
///             .ok_or("Invalid format, expected 'numerator/denominator'")?;
///         let numerator = numerator.parse().map_err(|_| "Invalid numerator")?;
///         let denominator = denominator.parse().map_err(|_| "Invalid denominator")?;
///         Ok(Fraction { numerator, denominator })
///     }
/// }
///
/// let parsed = sscanf::sscanf!("-10/3", "{Fraction}").unwrap();
/// assert_eq!(parsed, Fraction { numerator: -10, denominator: 3 });
/// ```
/// This approach requires manually specifying the regex for the type and implementing the [`FromStr`] trait for the
/// type in order to parse it. This setup is especially useful in cases where `FromStr` is already implemented for the
/// type, though even then it would be more efficient to use the derive macro, since it can directly extract the fields
/// from the initial match rather than having to parse and validate the string again.
///
/// ### Manually implementing the [`FromScanf`] trait
/// ```
/// use sscanf::from_scanf::{format_options::FormatOptions, FromScanf, FromScanfParser, RegexSegment};
///
/// #[derive(Debug, PartialEq)]
/// struct Fraction {
///     numerator: isize,
///     denominator: usize,
/// }
///
/// impl<'input> FromScanf<'input> for Fraction {
///     type Parser = FractionParser; // Custom parser type
///
///     fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
///         // Create a regex segment that matches a possibly negative numerator and a denominator
///         // and captures each in a separate capture group.
///         let regex = RegexSegment::new(r"([+-]?\d+)/(\d+)");
///         (regex, FractionParser)
///     }
/// }
///
/// struct FractionParser;
///
/// impl<'input> FromScanfParser<'input, Fraction> for FractionParser {
///     fn parse(&self, full_match: &'input str, sub_matches: &[Option<&'input str>]) -> Option<Fraction> {
///         assert_eq!(sub_matches.len(), 2,
///             "FractionParser expected 2 matches, got {}. Are there any custom types with incorrect FromScanf implementations?", sub_matches.len()
///         );
///         // Extract the values from the sub_matches.
///         let numerator = sub_matches[0].unwrap().parse().ok()?;
///         let denominator = sub_matches[1].unwrap().parse().ok()?;
///         Some(Fraction { numerator, denominator })
///     }
/// }
///
/// let parsed = sscanf::sscanf!("-10/3", "{Fraction}").unwrap();
/// assert_eq!(parsed, Fraction { numerator: -10, denominator: 3 });
/// ```
///
/// **Alternatively, using the provided `SubType` type:**
/// ```
/// use sscanf::from_scanf::{format_options::FormatOptions, FromScanf, FromScanfParser, RegexSegment, SubType};
///
/// #[derive(Debug, PartialEq)]
/// struct Fraction {
///     numerator: isize,
///     denominator: usize,
/// }
///
/// impl<'input> FromScanf<'input> for Fraction {
///     type Parser = FractionParser<'input>; // Custom parser type
///
///     fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
///         let (numerator_regex, numerator_parser) = SubType::<isize>::new(format);
///         let (denominator_regex, denominator_parser) = SubType::<usize>::new(format);
///         let regex = RegexSegment::new(&format!("{numerator_regex}/{denominator_regex}"));
///         (regex, FractionParser { numerator_parser, denominator_parser })
///     }
/// }
///
/// struct FractionParser<'input> {
///     numerator_parser: SubType<'input, isize>,
///     denominator_parser: SubType<'input, usize>,
/// }
///
/// impl<'input> FromScanfParser<'input, Fraction> for FractionParser {
///     fn parse(&self, full_match: &'input str, mut sub_matches: &[Option<&'input str>]) -> Option<Fraction> {
///         let numerator = self.numerator_parser.parse(&mut sub_matches)?;
///         let denominator = self.denominator_parser.parse(&mut sub_matches)?;
///         Some(Fraction { numerator, denominator })
///     }
/// }
///
/// let parsed = sscanf::sscanf!("-10/3", "{Fraction}").unwrap();
/// assert_eq!(parsed, Fraction { numerator: -10, denominator: 3 });
/// ```
///
/// This example demonstrates how to manually implement the [`FromScanf`] trait for a custom type. Just like in the
/// `FromScanfSimple` case, it requires two steps: Defining the regex to match against and implementing the parser
/// that extracts the values from the regex matches.
///
/// The key difference is the presence of "capture groups" (the round brackets in the regex). These groups allow you to
/// extract specific parts of the match, which is especially useful for complex types consisting of multiple parts like
/// structs or tuples. The `FromScanfParser` trait provides a way to handle these capture groups in the form of the
/// `sub_matches` parameter in the `parse` method.
///
/// Since the pattern of having inner types is quite common, the `SubType` type is provided to simplify the
/// implementation. It automatically handles the regex creation and parsing of the inner types, allowing you to
/// focus on the outer type. The code in the second example is actually nearly identical to what the derive macro
/// generates, albeit simplified and formatted for clarity.
pub trait FromScanf<'input>: Sized {
    /// Actual parser implementation.
    ///
    /// If you just want to parse with [`FromStr`], set this to `()`. That will become the default type once
    /// [Rust Issue 29661](https://github.com/rust-lang/rust/issues/29661) is resolved.
    type Parser: FromScanfParser<'input, Self>;

    /// TODO:
    ///
    /// Note: Composite types (e.g. structs, tuples) tend to pass unused format options to their subtypes. So your type
    /// might receive a format option that it doesn't know how to handle. In this case, you can just ignore it.
    #[track_caller]
    fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser);
}

/// A trait for the actual parser implementation.
///
/// Note that there is [a default implementation](#impl-FromScanfParser<'input,+T>-for-()) for a parser that uses
/// [`FromStr`] by using `()` as the parser type.
pub trait FromScanfParser<'input, T>: Sized {
    /// Parse the regex matches into the type.
    ///
    /// The `full_match` is the entire match for this type. This is what most types will use for parsing.
    ///
    /// The `sub_matches` are any capture groups within the regex. `sub_matches` are usually only used by inner
    /// types, and [`SubType`] is a convenient way to handle them. See [`SubType::parse`] for more information.
    fn parse(&self, full_match: &'input str, sub_matches: &[Option<&'input str>]) -> Option<T>;
}

/// A simplified version of [`FromScanf`] that is easier to implement and works for most types.
///
/// Manually implementing the non-simple [`FromScanf`] is only necessary if
/// - the type is a struct or tuple that contains other types (see [the derive macro](crate::macros::FromScanf))
/// - the type contains generic parameters (e.g. [`Option<T>`](trait.FromScanf.html#impl-FromScanf<'input>-for-Option<T>))
/// - the type wants to borrow from the input string (e.g. [`&'input str`](trait.FromScanf.html#impl-FromScanf<'input>-for-%26str))
pub trait FromScanfSimple: Sized + FromStr {
    /// The regex for the type.
    const REGEX: &'static str;
}

impl<T: FromScanfSimple + FromStr> FromScanf<'_> for T {
    type Parser = ();

    fn create_parser(_: &FormatOptions) -> (RegexSegment, Self::Parser) {
        (RegexSegment::new(T::REGEX), ())
    }
}

impl<'input, T: FromScanf<'input> + FromStr> FromScanfParser<'input, T> for () {
    /// Parses the full match using the type's [`FromStr`] implementation.
    fn parse(&self, full_match: &'input str, _: &[Option<&'input str>]) -> Option<T> {
        full_match.parse().ok()
    }
}
