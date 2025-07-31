#![allow(unused)]
//! Utilities for the macros. These elements are public but doc_hidden

use crate::match_tree::{self, MatchTree, MatchTreeIndex};

/// Wrapper around `const_format::concatcp!` so that the dependency is not part of our public API.
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
                match_tree::fill_index(&hir, &mut match_tree_index);

                Ok((regex, match_tree_index))
            })
            .as_ref()
            .expect("sscanf: Failed to compile regex"); // This will panic at the call site of `sscanf!`
    }

    pub fn parse_captures<'input, T>(
        &self,
        input: &'input str,
        f: impl FnOnce(&MatchTree<'_, 'input>) -> Option<T>,
    ) -> Option<T> {
        let (regex, index) = self.regex.get().unwrap().as_ref().unwrap();
        let mut captures = regex.create_captures();
        regex.captures(input, &mut captures);
        let match_tree = MatchTree::new(
            index,
            &captures,
            input,
            captures.get_group(0)?,
            match_tree::Context::Named("sscanf").into(),
        );
        f(&match_tree)
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
