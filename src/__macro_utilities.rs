#![allow(unused)]
//! Utilities for the macros. These elements are public but doc_hidden

use crate::advanced::*;

/// Wrapper around `const_format::concatcp!` so that the dependency is not part of our public API.
macro_rules! concat_str {
    ( $( $parts:expr ),* ) => {
        const_format::concatcp!( $( $parts ),* )
    };
}

/// Wrapper around regex so that the dependency is not part of our public API.
///
/// Also takes care of the whole "only compile regex once" thing.
pub struct Parser {
    regex: std::sync::OnceLock<ParserMeta>,
}

struct ParserMeta {
    regex: Result<regex_automata::meta::Regex, String>,
    match_tree_index: MatchTreeIndex,
    hir: regex_syntax::hir::Hir,
}

impl Parser {
    /// Creates an empty `WrappedRegex`. Callable in `const` context.
    #[allow(
        clippy::new_without_default,
        reason = "This is a const constructor. There is no reason to have Default for this struct."
    )]
    pub const fn new() -> Self {
        Self {
            regex: std::sync::OnceLock::new(),
        }
    }

    #[track_caller]
    pub fn assert_compiled(&self, get_matcher: impl FnOnce() -> Matcher) {
        let regex = self.regex.get_or_init(|| {
            let matcher = get_matcher();

            let mut capture_index = 0;
            let hir = matcher.compile(&mut capture_index);

            let mut match_tree_index = MatchTreeIndex {
                index: 0,
                children: Vec::new(),
            };
            fill_index(&hir, &mut match_tree_index);

            if hir.properties().explicit_captures_len() != capture_index {
                let error = format!(
                    "sscanf: Matcher has mismatched number of capture groups! Expected {}, got {}",
                    capture_index,
                    hir.properties().explicit_captures_len()
                );
                return ParserMeta {
                    regex: Err(error),
                    match_tree_index,
                    hir,
                };
            }

            let regex = regex_automata::meta::Regex::builder()
                .build_from_hir(&hir)
                .map_err(|e| format!("sscanf: Failed to compile regex: {e}"));

            ParserMeta {
                regex,
                match_tree_index,
                hir,
            }
        });
        if let Err(err) = regex.regex.as_ref() {
            panic!("{err}"); // This will panic at the call site of `sscanf!`
        }
    }

    pub fn parse_captures<'input, T>(
        &self,
        input: &'input str,
        f: impl FnOnce(&MatchTree<'_, 'input>) -> Option<T>,
    ) -> Option<T> {
        let meta = self.regex.get().unwrap();
        let regex = meta.regex.as_ref().unwrap();
        let mut captures = regex.create_captures();
        regex.captures(input, &mut captures);
        let match_tree = MatchTree::new(
            &meta.match_tree_index,
            &captures,
            input,
            captures.get_group(0)?,
            Context::Named("sscanf").into(),
        );
        f(&match_tree)
    }
}
impl std::fmt::Debug for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.regex.get() {
            Some(meta) => f
                .debug_struct("WrappedRegex")
                .field("regex", &meta.regex)
                .field("match_tree", &meta.match_tree_index)
                .field("hir", &meta.hir)
                .finish(),
            None => write!(f, "<Not compiled yet>"),
        }
    }
}
impl std::fmt::Display for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.regex.get() {
            Some(meta) => meta.hir.fmt(f),
            None => write!(f, "<Not compiled yet>"),
        }
    }
}
