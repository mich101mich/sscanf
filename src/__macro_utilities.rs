#![allow(unused)]
//! Utilities for the macros. These elements are public but doc_hidden

use crate::MatchTree;

#[macro_export]
macro_rules! concat_str {
    ( $( $parts:expr ),* ) => {
        const_format::concatcp!( $( $parts ),* )
    };
}
pub use concat_str;

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
