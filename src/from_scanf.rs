use std::str::FromStr;

mod format_options;
mod impls;
mod regex_segment;
mod sub_type;

pub use format_options::*;
pub use regex_segment::*;
pub use sub_type::*;

/// TODO:
pub trait FromScanf<'input>: Sized {
    /// Actual parser implementation
    ///
    /// If you just want to parse with [`FromStr`], set this to `()`.
    type Parser: FromScanfParser<'input, Self>;

    /// TODO:
    ///
    /// Note: Composite types (e.g. structs, tuples) tend to pass unused format options to their subtypes. So your type
    /// might receive a format option that it doesn't know how to handle. In this case, you can just ignore it.
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
