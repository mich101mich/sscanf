//! Module containing the [`Sscanf`] struct.

use std::borrow::Cow;

use super::from_scanf::*;
use regex::{Captures, Regex};

/// An intermediate type to hold the state of one `sscanf!` format and type specification to allow applying it to
/// multiple inputs, either fully or as an iterator.
pub struct Sscanf<'input, Tuple: 'input> {
    full_regex: Regex,
    iter_regex: Option<Regex>,
    raw_regex: Cow<'static, str>,
    // type erasure for the intermediate type. See the documentation for `Sscanf::new` for more details.
    parser: Box<dyn Fn(Captures<'input>) -> Option<Tuple> + 'input>,
}

impl<'input, Tuple: 'input> Sscanf<'input, Tuple> {
    /// Parses an input string. Note that the entire input string from start to end has to match the sscanf format.
    pub fn parse(&self, input: &'input str) -> Option<Tuple> {
        (self.parser)(self.full_regex.captures(input)?)
    }

    /// Creates an iterator over all non-overlapping matches in the input string. Note that this implies that the
    /// sscanf format **does not** match the entire input string, but only parts of it, since the iterator would
    /// otherwise only yield at most one element.
    ///
    /// Though, on the other hand, using this creates an opportunity to have a partial match, which is not possible
    /// with the [`Sscanf::parse`] method.
    ///
    /// # Caveats
    /// The parsing procedure consists of two steps: Finding a match and parsing it. For most other applications, a
    /// failure in either will result in `None` being returned, so it doesn't matter which one failed. However, in this
    /// case, matches have to be non-overlapping, so if a match is found but parsing fails, the iterator will skip the
    /// entire match and continue searching for the next one. This creates an unfortunate but rare edge case where an
    /// actual match is skipped because a partially overlapping incorrect match was found first.
    ///
    /// This crate tries to have all of its [`FromScanf`] implementations be as exact as is possible within reason,
    /// but there are bound to be edge cases. If you encounter one, please
    /// [open an issue](https://github.com/mich101mich/sscanf/issues)
    pub fn parse_iter<'parser: 'input>(
        &'parser mut self,
        input: &'input str,
    ) -> SscanfParseIter<'parser, 'input, Tuple> {
        let iter_regex = self.iter_regex.get_or_insert_with(|| {
            Regex::new(&self.raw_regex).expect("sscanf: Failed to create regex")
        });
        SscanfParseIter {
            parser: &self.parser,
            iter: iter_regex.captures_iter(input),
        }
    }

    /// Creates a new `Sscanf` instance using an intermediate type.
    ///
    /// <div class="warning">
    ///
    /// The main way to create an instance of `Sscanf` is through the [`sscanf_parser!`](crate::macros::sscanf_parser)
    /// macro. This function is exposed so that the macro can use it and for completeness.
    ///
    /// </div>
    ///
    /// This function requires a template type `T` that acts as an intermediate type for parsing the input. The reason
    /// for that is that `sscanf!` (and thus `Sscanf`) are supposed to return a tuple of the placeholder types, but
    /// the `FromScanf` trait is not implemented for those tuples, not could it be, since the format string is
    /// different for each call to `sscanf!`. Thus, an intermediate type is used to parse the input and then
    /// convert it into a tuple.
    ///
    /// When using the `sscanf_parser!` macro (or any of the other macros internally), a local type is generated
    /// that implements the `FromScanf` trait and can be converted into a tuple using the [`Into`] trait. This type
    /// is then used as the template type `T` for this function.
    ///
    /// Hence why it is technically possible to call this function directly, but it is not recommended,
    pub fn new<T>() -> Self
    where
        T: FromScanf<'input> + Into<Tuple> + 'input,
    {
        let (regex, parser) = T::create_parser(&Default::default());

        let parser = move |captures: Captures<'input>| {
            let matches: Vec<_> = captures.iter().map(|m| m.map(|m| m.as_str())).collect();

            // from Captures::iter documentation: "The iterator always yields at least one matching group: the first group (at index `0`) with no name."
            // => it is safe to unwrap the first element
            let (full_match, sub_matches) = matches.split_first().unwrap();
            Some(parser.parse(full_match.unwrap(), sub_matches)?.into())
        };

        let raw_regex = regex.into_raw_regex();
        Self {
            full_regex: Regex::new(&format!("^{}$", raw_regex))
                .expect("sscanf: Failed to create regex"),
            iter_regex: None,
            raw_regex,
            parser: Box::new(parser),
        }
    }

    /// Returns the raw regex used for this `Sscanf` instance.
    ///
    /// Note that when calling [`Sscanf::parse`], the regex is prefixed with `^` and suffixed with `$`, so it matches
    /// the entire input string.
    pub fn raw_regex(&self) -> String {
        self.raw_regex.to_string()
    }
}

/// The iterator returned by [`Sscanf::parse_iter`].
pub struct SscanfParseIter<'parser, 'input: 'parser, Tuple> {
    parser: &'parser (dyn Fn(Captures<'input>) -> Option<Tuple> + 'input),
    iter: ::regex::CaptureMatches<'parser, 'input>,
}

impl<'input, Tuple: FromScanf<'input>> Iterator for SscanfParseIter<'_, 'input, Tuple> {
    type Item = Tuple;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.by_ref().find_map(self.parser)
    }
}
