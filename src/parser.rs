use crate::{FromScanf, advanced::*};

use regex_syntax::hir::{Hir, Look};

/// A parser that can be reused to parse multiple inputs.
///
/// This type is typically created via [`sscanf_parser`](crate::sscanf_parser). Though it can be constructed directly,
/// for example for custom types.
///
/// ### Caveats
/// - Types that borrow from the input (like `&str`) must share the same lifetime across all inputs. To parse
///   inputs with different lifetimes, create multiple `Parser` instances.
pub struct Parser<'input, T> {
    regex: regex_automata::meta::Regex,
    captures: regex_automata::util::captures::Captures,
    match_tree_template: MatchTreeTemplate,
    parse_fn: Box<dyn FnMut(Match<'_, 'input>) -> Option<T>>,
}

impl<'input, T> Parser<'input, T> {
    /// Create a new parser around type `T`.
    ///
    /// For a single input, use the convenience function [`sscanf::parse`](crate::parse).
    ///
    /// This type is mostly used if you need to cache the parser for multiple uses.
    pub fn new() -> Self
    where
        T: FromScanf<'input>,
    {
        Self::with_options(Default::default())
    }

    /// Create a new parser around type `T` with the given format options.
    pub fn with_options(options: FormatOptions) -> Self
    where
        T: FromScanf<'input>,
    {
        let matcher = T::get_matcher(&options);
        let parse_fn = move |match_tree: Match<'_, 'input>| T::from_match(match_tree, &options);
        Self::from_matcher(matcher, parse_fn)
    }

    /// Create a parser directly from a `Matcher`.
    ///
    /// Prefer [`Parser::new`], which constructs the matcher and parser from `T` to ensure consistency.
    /// This method is exposed for situations without a single `T`, like the `sscanf!` macro.
    pub fn from_matcher(
        matcher: Matcher,
        parse_fn: impl FnMut(Match<'_, 'input>) -> Option<T> + 'static,
    ) -> Self {
        // We need to re-index the capture groups. Capture group 0 is the whole match, so our matchers
        // should start at 1. However, since our outermost Matcher is itself the whole match, we assign it
        // to group 0 but then remove it again after compilation.
        let mut capture_index = 0;
        let (capture, match_tree_template) = matcher.compile(&mut capture_index);

        // Remove the outermost capture group since it is identical to the whole match.
        let hir = *capture.sub;
        capture_index -= 1;

        // Ensure we match the entire input string (equivalent to adding `^` and `$` around the regex)
        let hir = Hir::concat(vec![Hir::look(Look::Start), hir, Hir::look(Look::End)]);

        if hir.properties().explicit_captures_len() != capture_index {
            // Since we manually re-indexed the capture groups, this should never happen
            panic!(
                "sscanf: Internal Error: Matcher has mismatched number of capture groups! Expected {capture_index}, got {}",
                hir.properties().explicit_captures_len()
            );
        }

        let regex = regex_automata::meta::Regex::builder()
            .build_from_hir(&hir)
            .expect("sscanf: Failed to compile regex from Matcher");
        // Since build_from_hir doesn't need to parse the regex from text, there are only very few reasons for it to
        // fail. These are:
        // - Size limits being exceeded (hard error, usually from terrible/malicious custom regex)
        // - Conflicting capture indices (we index them ourselves, so this should never happen)
        // - Internal errors in regex-automata (the regex crate is very well tested, so this should never happen)

        let captures = regex.create_captures();

        Self {
            regex,
            captures,
            match_tree_template,
            parse_fn: Box::new(parse_fn),
        }
    }

    /// Parse the given input string into a value of type `T`.
    pub fn parse(&mut self, input: &'input str) -> Option<T> {
        self.regex.captures(input, &mut self.captures);
        let match_tree = Match::new(
            &self.match_tree_template,
            &self.captures,
            input,
            self.captures.get_group(0)?,
            Context::Root.into(),
        );
        (self.parse_fn)(match_tree)
    }
}

impl<'input, T: FromScanf<'input>> Default for Parser<'input, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Debug for Parser<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("Parser<{}>", std::any::type_name::<T>()))
            .field("regex", &self.regex)
            .field("match_tree_template", &self.match_tree_template)
            .finish()
    }
}
