use super::*;

/// A segment of a regex.
///
/// This type is needed to keep track of which type's regex contains how many capture groups, since `sscanf` needs to
/// pass the capture matches back to the corresponding type's [`FromScanfParser::parse`] method.
///
/// A capture group is a set of parentheses in a regex `(...)`, which is not escaped by a backslash `\(...\)` or the
/// non-capturing group indicator `(?:...)`. If you are absolutely certain that you know the exact number of capture
/// groups in your regex, you can call [`RegexSegment::from_known`]. Otherwise, just call [`RegexSegment::new`].
#[derive(Debug, Clone)]
pub struct RegexSegment {
    pub(crate) regex: String,
    pub(crate) num_capture_groups: usize,
}

impl RegexSegment {
    /// Creates a new regex segment with the given regex.
    ///
    /// # Panics
    /// Panics if the regex is invalid.
    pub fn new(regex: &str) -> Self {
        Self::from_known(regex, count_capture_groups(regex))
    }

    /// Creates a new regex segment where the number of capture groups is already known.
    ///
    /// If the given number of capture groups does not match the actual number of capture groups in the regex, panics
    /// will happen at some point during the parsing process, or they won't at it will simply return `None` without
    /// telling you what went wrong. The exact behavior depends the error and the surrounding types, so it's best not
    /// to risk it unless you are absolutely certain. If you aren't, just use [`RegexSegment::new`].
    pub fn from_known(regex: &str, num_capture_groups: usize) -> Self {
        Self {
            regex: format!("({regex})"), // add a capture group around the whole regex for the full match
            num_capture_groups: num_capture_groups + 1, // +1 because we just added a capture group
        }
    }

    /// Returns the number of capture groups in the regex.
    pub fn num_capture_groups(&self) -> usize {
        self.num_capture_groups
    }

    /// Returns the regex string.
    pub fn regex(&self) -> &str {
        &self.regex
    }

    /// Internal utility for using a custom regex
    #[track_caller]
    pub(crate) fn _maybe_replace_with<T>(&mut self, custom_regex: Option<Self>) {
        let Some(custom_regex) = custom_regex else {
            return;
        };
        if self.num_capture_groups != custom_regex.num_capture_groups {
            panic!(
                "Custom regex of {} must have the same number of capture groups as the default regex
Default regex ({} capture groups): {}
 Custom regex ({} capture groups): {}",
                std::any::type_name::<T>(),
                self.num_capture_groups,
                self.regex,
                custom_regex.num_capture_groups,
                custom_regex.regex
            );
        }
        self.regex = custom_regex.regex;
        self.num_capture_groups = custom_regex.num_capture_groups;
    }

    /// Internal utility for using a custom regex
    #[track_caller]
    pub(crate) fn _with_format<T, P: Default>(self, format: FormatOptions) -> (Self, P) {
        let Some(custom_regex) = format.regex else {
            return (self, P::default());
        };
        if self.num_capture_groups != custom_regex.num_capture_groups {
            panic!(
                "Custom regex of {} must have the same number of capture groups as the default regex
Default regex ({} capture groups): {}
 Custom regex ({} capture groups): {}",
                std::any::type_name::<T>(),
                self.num_capture_groups,
                self.regex,
                custom_regex.num_capture_groups,
                custom_regex.regex
            );
        }
        (custom_regex, P::default())
    }
}

impl std::fmt::Display for RegexSegment {
    /// Writes the regex string. Useful when combining the regex from an inner type
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.regex)
    }
}

pub(crate) fn count_capture_groups(regex: &str) -> usize {
    let mut queue = vec![];
    let hir = regex_syntax::parse(regex)
        .unwrap_or_else(|err| panic!("Failed to parse regex '{}': {}", regex, err));
    queue.push(hir);

    let mut count = 0;
    while let Some(hir) = queue.pop() {
        match hir.into_kind() {
            regex_syntax::hir::HirKind::Capture(capture) => {
                count += 1;
                queue.push(*capture.sub);
            }
            regex_syntax::hir::HirKind::Repetition(repetition) => {
                queue.push(*repetition.sub);
            }
            regex_syntax::hir::HirKind::Concat(vec)
            | regex_syntax::hir::HirKind::Alternation(vec) => {
                queue.extend(vec.into_iter());
            }
            _ => {}
        }
    }
    count
}

#[cfg(test)]
pub(crate) mod tests {
    use super::RegexSegment;

    // Traits that are only used in tests
    impl PartialEq<RegexSegment> for RegexSegment {
        fn eq(&self, other: &RegexSegment) -> bool {
            self.regex == other.regex && self.num_capture_groups == other.num_capture_groups
        }
    }
    impl PartialEq<&str> for RegexSegment {
        fn eq(&self, other: &&str) -> bool {
            self.regex == format!("({other})") && self.num_capture_groups == 1
        }
    }

    #[test]
    fn dev_added_tests_for_count_capture_groups() {
        panic!("No he didn't");
    }
}
