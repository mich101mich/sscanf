#![allow(unused)]
//! Utilities for the macros. These elements are public but doc_hidden

#[doc(hidden)]
pub static EXPECT_NEXT_HINT: &str = r#"sscanf: Invalid number of capture groups in regex.
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid forming a capture group like this:
    "  (  )  "  =>  "  (?:  )  ""#;

#[doc(hidden)]
pub static EXPECT_CAPTURE_HINT: &str = r#"sscanf: Non-optional capture group marked as optional.
This is either a problem with a custom regex or FromScanf implementation or an internal error."#;

#[doc(hidden)]
pub static WRONG_CAPTURES_HINT: &str = r#"
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid forming a capture group like this:
    "  (  )  "  =>  "  (?:  )  "
"#;

/// Wrapper around regex so that the dependency is not part of our public API.
///
/// Also takes care of the whole "only compile regex once" thing.
pub struct WrappedRegex {
    /// The regex itself, wrapped in a `OnceLock` to ensure it is only compiled once.
    ///
    /// Note the weird design with the Result in a OnceLock. This is because if the compilation of the regex fails,
    /// we want to panic (regex is known at compile time => it is a programming error), and we want the panic location
    /// to be at the call site of `sscanf!`. So we need some initialization that guarantees the "only compile once"
    /// behavior, and can be put in a track_caller context (so no closure).
    ///
    /// The only alternative would be our own synchronization scheme or a mutex, but that would be overkill considering
    /// that 99.99% of use cases only invoke sscanf once, and even those that invoke it multiple times do so from the
    /// same thread. A OnceLock is only really used here so that WrappedRegex can be put in a static variable.
    regex: std::sync::OnceLock<Result<regex::Regex, regex::Error>>,
    regex_str: &'static str,
}
impl WrappedRegex {
    /// Creates an empty `WrappedRegex`. Callable in `const` context.
    pub const fn new(regex: &'static str) -> Self {
        Self {
            regex: std::sync::OnceLock::new(),
            regex_str: regex,
        }
    }

    #[track_caller]
    pub fn assert_compiled(&self, num_captures: usize) {
        let regex = self
            .regex
            .get_or_init(|| regex::Regex::new(self.regex_str))
            .as_ref()
            .expect("sscanf: Failed to compile regex"); // This will panic at the call site of `sscanf!`

        let actual_captures = regex.captures_len();
        if actual_captures != num_captures {
            panic!(
                "sscanf: Regex has {actual_captures} capture groups, but {num_captures} were expected."
            );
        }
    }

    pub fn captures<'input>(&self, input: &'input str) -> Option<Vec<Option<&'input str>>> {
        let regex = self.regex.get().unwrap().as_ref().unwrap();
        let captures = regex.captures(input)?;
        let mut iter = captures.iter();
        // Regex documentation states:
        // "The iterator always yields at least one matching group: the first group (at index `0`) with no name."
        // So we can safely unwrap the first element.
        iter.next().unwrap().unwrap().as_str(); // skip the whole match
        let sub_matches = iter.map(|m| m.map(|m| m.as_str())).collect();
        Some(sub_matches)
    }
}

/// Counts the number of capture groups in a regex string.
///
/// Valid capture group: "...(...)...".
/// Escaped: "...\(...)..." or "...(?:...)...".
///
/// Note that this function assumed that the input is a valid regex string.
pub const fn count_sub_captures(input: &'static str) -> usize {
    let mut count = 0;
    let mut escaped = false;

    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        match bytes[i] {
            b'(' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'?' {
                    // non-capturing group
                    continue;
                }
                count += 1;
            }
            b'\\' => escaped = true,
            _ => {}
        }
        i += 1;
    }

    count
}
