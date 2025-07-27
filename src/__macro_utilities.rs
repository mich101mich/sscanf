#![allow(unused)]
//! Utilities for the macros. These elements are public but doc_hidden

use crate::MatchTree;

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
    regex: std::sync::OnceLock<Result<(regex_automata::meta::Regex, MatchTreeIndex), String>>,
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
    pub fn assert_compiled(&self) {
        let regex = self
            .regex
            .get_or_init(|| {
                let hir = regex_syntax::parse(self.regex_str).map_err(|e| e.to_string())?;

                let regex = regex_automata::meta::Regex::builder()
                    .build_from_hir(&hir)
                    .map_err(|e| e.to_string())?;

                let mut match_tree_index = MatchTreeIndex {
                    index: 0,
                    children: Vec::new(),
                };
                create_match_tree_index(&hir, &mut match_tree_index);

                Ok((regex, match_tree_index))
            })
            .as_ref()
            .expect("sscanf: Failed to compile regex"); // This will panic at the call site of `sscanf!`
    }

    pub fn captures<'input>(&self, input: &'input str) -> Option<MatchTree<'input>> {
        let (regex, index) = self.regex.get().unwrap().as_ref().unwrap();
        let mut captures = regex.create_captures();
        regex.captures(input, &mut captures);
        index.create_match_tree(&captures, input)
    }
}
impl std::fmt::Debug for WrappedRegex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WrappedRegex")
            .field("regex", &self.regex_str)
            .finish()
    }
}
impl std::fmt::Display for WrappedRegex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.regex_str.fmt(f)
    }
}

/// The source structure of a MatchTree, consisting of only the indices in the capture group list.
struct MatchTreeIndex {
    index: usize,
    children: Vec<MatchTreeIndex>,
}
impl MatchTreeIndex {
    pub fn create_match_tree<'input>(
        &self,
        captures: &regex_automata::util::captures::Captures,
        input: &'input str,
    ) -> Option<MatchTree<'input>> {
        let full = captures.get_group(self.index)?;
        let inner = self
            .children
            .iter()
            .map(|child| child.create_match_tree(captures, input))
            .collect();
        Some(MatchTree {
            full: &input[full],
            inner,
        })
    }
}

fn create_match_tree_index(hir: &regex_syntax::hir::Hir, out: &mut MatchTreeIndex) {
    match hir.kind() {
        regex_syntax::hir::HirKind::Capture(capture) => {
            let mut child = MatchTreeIndex {
                index: capture.index as usize,
                children: Vec::new(),
            };
            create_match_tree_index(&capture.sub, &mut child);
            out.children.push(child);
        }

        regex_syntax::hir::HirKind::Repetition(repetition) => {
            create_match_tree_index(&repetition.sub, out);
        }
        regex_syntax::hir::HirKind::Concat(hirs) => {
            for sub_hir in hirs {
                create_match_tree_index(sub_hir, out);
            }
        }
        regex_syntax::hir::HirKind::Alternation(hirs) => {
            for sub_hir in hirs {
                create_match_tree_index(sub_hir, out);
            }
        }
        _ => {}
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
