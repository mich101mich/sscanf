#![allow(unused)]
//! Utilities for the macros. These elements are public but doc_hidden

use regex_syntax::hir::{Hir, HirKind, Look};

use crate::advanced::*;

/// Wrapper around regex so that the dependency is not part of our public API.
///
/// Also takes care of the whole "only compile regex once" thing.
pub struct Parser {
    regex: std::sync::OnceLock<ParserMeta>,
}

struct ParserMeta {
    regex: Result<regex_automata::meta::Regex, String>,
    match_tree_template: MatchTreeTemplate,
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

            // We need to re-index the capture groups. Capture group 0 is the whole match, so our matchers
            // should start at 1. However, since our outermost Matcher is itself the whole match, we assign it
            // to group 0 but then remove it again after compilation.
            let mut capture_index = 0;
            let (hir, match_tree_template) = match matcher.compile(&mut capture_index) {
                Ok(hir) => hir,
                Err(err) => panic!("{err}"),
            };

            // Remove the outermost capture group since it is identical to the whole match.
            let hir = match hir.into_kind() {
                HirKind::Capture(capture) => *capture.sub,
                _ => panic!("sscanf: Internal error: Matcher did not compile to a capture group!"),
            };
            capture_index -= 1;

            let hir = Hir::concat(vec![Hir::look(Look::Start), hir, Hir::look(Look::End)]);

            if hir.properties().explicit_captures_len() != capture_index {
                let error = format!(
                    "sscanf: Matcher has mismatched number of capture groups! Expected {capture_index}, got {}",
                    hir.properties().explicit_captures_len()
                );
                return ParserMeta {
                    regex: Err(error),
                    match_tree_template,
                };
            }

            let regex = regex_automata::meta::Regex::builder()
                .build_from_hir(&hir)
                .map_err(|e| format!("sscanf: Failed to compile regex: {e}"));

            ParserMeta {
                regex,
                match_tree_template,
            }
        });
        if let Err(err) = regex.regex.as_ref() {
            panic!("{err}"); // This will panic at the call site of `sscanf!`
        }
    }

    pub fn parse_captures<'input, T>(
        &self,
        input: &'input str,
        f: impl FnOnce(MatchTree<'_, 'input>) -> Option<T>,
    ) -> Option<T> {
        let meta = self.regex.get().unwrap();
        let regex = meta.regex.as_ref().unwrap();
        let mut captures = regex.create_captures();
        regex.captures(input, &mut captures);
        let match_tree = MatchTree::new(
            &meta.match_tree_template,
            &captures,
            input,
            captures.get_group(0)?,
            Context::Named("sscanf").into(),
        );
        f(match_tree)
    }
}
impl std::fmt::Debug for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.regex.get() {
            Some(meta) => f
                .debug_struct("WrappedRegex")
                .field("regex", &meta.regex)
                .field("match_tree", &meta.match_tree_template)
                .finish(),
            None => write!(f, "WrappedRegex{{ ...not compiled yet... }}"),
        }
    }
}
