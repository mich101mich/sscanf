use crate::{FromScanf, advanced::*};

use regex_syntax::hir::{Hir, HirKind, Look};

/// A parser type that allows parsing multiple inputs
#[derive(Debug)]
pub struct Parser<T> {
    regex: regex_automata::meta::Regex,
    match_tree_template: MatchTreeTemplate,
    format: Option<FormatOptions>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'input, T: FromScanf<'input>> Parser<T> {
    /// Create a new parser around a type `T`
    ///
    /// If you just need a parser to parse a single input, there is also a convenience shortcut at
    /// [`sscanf::parse`](crate::parse).
    ///
    /// This type is mostly used if you need to cache the parser for multiple uses.
    pub fn new() -> Self {
        Self::with_format(Default::default())
    }

    /// Create a new parser around a type `T` with the given format options
    pub fn with_format(format: FormatOptions) -> Self {
        let matcher = T::get_matcher(&format);
        Self::from_matcher_with_format(matcher, Some(format))
    }

    /// Tries to parse the given input string into a value of type `T`
    pub fn parse(&self, input: &'input str) -> Option<T> {
        self.parse_with(input, |match_tree| {
            if let Some(format) = &self.format {
                T::from_match_tree(match_tree, format)
            } else {
                T::from_match_tree(match_tree, &FormatOptions::default())
            }
        })
    }
}

impl<'input, T: FromScanf<'input>> Default for Parser<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Parser<T> {
    /// Directly create a parser from a `Matcher`.
    ///
    /// Note that you usually want to use [`Parser::new`] instead, which constructs the matcher
    /// from the type `T` and ensures that the matching and parsing uses the same type. This method is
    /// only exposed for situations where there is no single type `T`, like with the `sscanf!` macro.
    pub fn from_matcher(matcher: Matcher) -> Self {
        Self::from_matcher_with_format(matcher, None)
    }

    #[track_caller]
    fn from_matcher_with_format(matcher: Matcher, format: Option<FormatOptions>) -> Self {
        // We need to re-index the capture groups. Capture group 0 is the whole match, so our matchers
        // should start at 1. However, since our outermost Matcher is itself the whole match, we assign it
        // to group 0 but then remove it again after compilation.
        let mut capture_index = 0;
        let (hir, match_tree_template) = matcher.compile(&mut capture_index);

        // Remove the outermost capture group since it is identical to the whole match.
        let HirKind::Capture(capture) = hir.into_kind() else {
            // Matcher::compile returns a capture, so this should never happen
            panic!("sscanf: Internal error: Matcher did not compile to a capture group!");
        };
        capture_index -= 1;

        // Ensure we match the entire input string (equivalent to adding `^` and `$` around the regex)
        let hir = Hir::concat(vec![
            Hir::look(Look::Start),
            *capture.sub,
            Hir::look(Look::End),
        ]);

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

        Self {
            regex,
            match_tree_template,
            format,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Tries to parse the given input string into a value using a custom closure
    pub fn parse_with<'input>(
        &self,
        input: &'input str,
        f: impl FnOnce(MatchTree<'_, 'input>) -> Option<T>,
    ) -> Option<T> {
        let mut captures = self.regex.create_captures();
        self.regex.captures(input, &mut captures);
        let match_tree = MatchTree::new(
            &self.match_tree_template,
            &captures,
            input,
            captures.get_group(0)?,
            Context::Root.into(),
        );
        f(match_tree)
    }
}
