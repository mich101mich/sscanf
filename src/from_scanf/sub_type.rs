use super::*;

/// A convenience type for parsing inner types within a call to [`FromScanfParser::parse`].
///
/// This type can be stored as part of a type's [`FromScanf::Parser`].
pub struct SubType<'input, T: FromScanf<'input>> {
    parser: T::Parser,
    num_capture_groups: usize,
}

impl<'input, T: FromScanf<'input>> SubType<'input, T> {
    /// Creates a new `SubType` from the type's [`FromScanf::create_parser`] method. Also returns the [`RegexSegment`]
    /// for the outer type to use.
    pub fn new(mut format: FormatOptions) -> (RegexSegment, Self) {
        let custom_regex = format.regex.take();
        let (mut regex, parser) = T::create_parser(format);
        regex._maybe_replace_with::<Self>(custom_regex);
        let num_capture_groups = regex.num_capture_groups;
        (
            regex,
            Self {
                parser,
                num_capture_groups,
            },
        )
    }
}

impl<'input, T: FromScanf<'input>> SubType<'input, T> {
    /// Parses the matches of the regex into the type.
    ///
    /// This function should be called with the `sub_matches` slice of the parent type, and will automatically
    /// advance the slice to the end of this type's matches (which is usually the start of the next type's matches).
    pub fn parse(&self, matches: &mut &[Option<&'input str>]) -> Option<T> {
        let full_match = matches[0].unwrap();
        let sub_matches = &matches[1..=self.num_capture_groups];
        *matches = &matches[self.num_capture_groups + 1..];
        self.parser.parse(full_match, sub_matches)
    }
}
