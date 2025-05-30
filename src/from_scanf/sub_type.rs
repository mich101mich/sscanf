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
    pub fn new(format: &FormatOptions) -> (RegexSegment, Self) {
        let (regex, parser) = T::create_parser(format);
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
    #[track_caller]
    pub fn parse(&self, matches: &mut &[Option<&'input str>]) -> Option<T> {
        // ideally I would want this to be `split_at_checked` followed by a `panic!` if the length is wrong, but
        // that method is much newer than our MSRV, so we use `assert!` + `split_at` instead
        assert!(
            matches.len() >= self.num_capture_groups,
            "sscanf::SubType<{}> expected {} matches, got {}. Are there any custom types with incorrect FromScanf implementations?",
            std::any::type_name::<T>(),
            self.num_capture_groups,
            matches.len(),
        );
        let (taken, rest) = matches.split_at(self.num_capture_groups);
        *matches = rest;

        let Some((Some(full_match), sub_matches)) = taken.split_first() else {
            panic!(
                "sscanf::SubType<{}> expected a full match, got None. Is there an unescaped '?' in a custom regex?",
                std::any::type_name::<T>(),
            );
        };

        self.parser.parse(full_match, sub_matches)
    }
}
