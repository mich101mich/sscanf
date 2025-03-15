use crate::FromScanfParser;

use super::*;
use regex::Regex;

/// An intermediate type to hold the state of one `sscanf!` format and type specification to allow applying it to
/// multiple inputs or generating an iterator from one input.
pub struct Sscanf<'input, T: FromScanf<'input>> {
    full_regex: Option<Regex>,
    iter_regex: Option<Regex>,
    raw_regex: RegexSegment,
    parser: T::Parser,
}

impl<'input, T: FromScanf<'input>> Sscanf<'input, T> {
    /// Parses an input string. Note that the entire input string from start to end has to match the sscanf format.
    pub fn parse(&mut self, input: &'input str) -> Option<T> {
        let regex = self.raw_regex.regex();
        let full_regex = self
            .full_regex
            .get_or_insert_with(|| Regex::new(&format!("^{}$", regex)).unwrap());

        let sub_matches = full_regex
            .captures(input)?
            .iter()
            .map(|m| m.map(|m| m.as_str()))
            .collect::<Vec<_>>();

        self.parser.parse(input, &sub_matches)
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
    /// but there are bound to be edge cases. If you encounter one, please open an issue on the GitHub repository.
    pub fn parse_iter(&mut self, input: &'input str) -> SscanfParseIter<'_, 'input, T> {
        let regex = self.raw_regex.regex();
        let iter_regex = self
            .iter_regex
            .get_or_insert_with(|| Regex::new(regex).unwrap());

        SscanfParseIter {
            parser: &self.parser,
            iter: iter_regex.captures_iter(input),
        }
    }

    /// Creates a new `Sscanf` object. Mostly for internal use by the TODO: macro. Left public so that the macro can
    /// access it.
    #[doc(hidden)]
    pub fn new(format: FormatOptions) -> Self {
        let (raw_regex, parser) = T::create_parser(format);
        Self {
            full_regex: None,
            iter_regex: None,
            raw_regex,
            parser,
        }
    }
}

/// The iterator returned by [`Sscanf::parse_iter`].
pub struct SscanfParseIter<'parser, 'input, T: FromScanf<'input>> {
    parser: &'parser T::Parser,
    iter: ::regex::CaptureMatches<'parser, 'input>,
}

impl<'input, T: FromScanf<'input>> Iterator for SscanfParseIter<'_, 'input, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let parser = self.parser;
        self.iter.by_ref().find_map(|captures| {
            let sub_matches = captures
                .iter()
                .map(|m| m.map(|m| m.as_str()))
                .collect::<Vec<_>>();
            // from Captures::iter documentation: The iterator always yields at least one matching group: the first group (at index `0`) with no name.
            // => it is safe to unwrap the first element
            let (full_match, sub_matches) = sub_matches.split_first().unwrap();
            parser.parse(full_match.unwrap(), sub_matches)
        })
    }
}
