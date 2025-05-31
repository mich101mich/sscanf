use std::borrow::Cow;

/// A segment of a regex.
///
/// This type is needed to keep track of which type's regex contains how many capture groups, since `sscanf` needs to
/// pass the capture matches back to the corresponding type's [`FromScanfParser::parse`][super::FromScanfParser::parse] method.
///
/// A capture group is a set of parentheses in a regex `(...)`, which is not escaped by a backslash `\(...\)` or the
/// non-capturing group indicator `(?:...)`. If you are absolutely certain that you know the exact number of capture
/// groups in your regex, you can call [`RegexSegment::from_known`]. Otherwise, just call [`RegexSegment::new`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexSegment {
    pub(crate) regex: Cow<'static, str>, // NOTE: incomplete. another capture group will be added on use
    pub(crate) num_capture_groups: usize,
}

impl RegexSegment {
    /// Creates a new regex segment with the given regex.
    ///
    /// # Panics
    /// Panics if the regex is invalid.
    #[track_caller]
    pub fn new(regex: impl Into<Cow<'static, str>>) -> Self {
        let regex = regex.into();
        let num_capture_groups = count_capture_groups(&regex);
        Self::from_known(regex, num_capture_groups)
    }

    /// Creates a new regex segment where the number of capture groups is already known.
    ///
    /// If the given number of capture groups does not match the actual number of capture groups in the regex, panics
    /// will happen at some point during the parsing process, or they won't at it will simply return `None` without
    /// telling you what went wrong. The exact behavior depends the error and the surrounding types, so it's best not
    /// to risk it unless you are absolutely certain. If you aren't, just use [`RegexSegment::new`].
    pub fn from_known(regex: impl Into<Cow<'static, str>>, num_capture_groups: usize) -> Self {
        Self {
            regex: regex.into(),
            num_capture_groups: num_capture_groups + 1, // +1 because we add a capture group
        }
    }

    /// Returns the number of capture groups in the regex.
    pub fn num_capture_groups(&self) -> usize {
        self.num_capture_groups
    }

    /// Returns the regex string. Identical to the `Display` implementation.
    pub fn regex(&self) -> String {
        self.to_string()
    }

    /// Internal method to get the raw regex string without the extra capture group.
    pub(crate) fn into_raw_regex(self) -> Cow<'static, str> {
        self.regex
    }
}

impl std::fmt::Display for RegexSegment {
    /// Writes the regex string. Useful when combining the regex from an inner type
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.regex)
    }
}

#[track_caller]
pub(crate) fn count_capture_groups(regex: &str) -> usize {
    match regex_syntax::parse(regex) {
        Ok(t) => t.properties().explicit_captures_len(),
        Err(e) => panic!("Failed to parse regex '{regex}': {e}"),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_count_capture_groups() {
        assert_eq!(count_capture_groups("a"), 0);
        assert_eq!(count_capture_groups("a(b)c"), 1);
        assert_eq!(count_capture_groups("a(b(c))"), 2);
        assert_eq!(count_capture_groups("(a)?"), 1);
        assert_eq!(count_capture_groups("a(?:b(c))"), 1);
        assert_eq!(count_capture_groups("a(b(?:c))"), 1);
        assert_eq!(count_capture_groups("(a)|(b)"), 2);
        assert_eq!(count_capture_groups("(a)|b"), 1);
        assert_eq!(count_capture_groups("(a|b)"), 1);
        assert_eq!(count_capture_groups("((a)())(()())"), 6);
        assert_eq!(count_capture_groups("(?:(?:a)(?:))(?:(?:)(?:))"), 0);
        assert_eq!(
            count_capture_groups("\\(\\(a\\)\\(\\)\\)\\(\\(\\)\\(\\)\\)"),
            0
        );
    }
}
