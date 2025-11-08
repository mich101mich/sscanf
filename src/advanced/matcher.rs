use std::borrow::Cow;

use regex_syntax::hir::{Capture, Hir};

/// TODO:
#[derive(Debug, Clone)]
pub struct Matcher {
    inner: MatcherType,
}

impl Matcher {
    /// Create a new matcher from a regex string.
    #[track_caller]
    pub fn from_regex(regex: &str) -> Self {
        let inner = regex_syntax::parse(regex).expect("Failed to parse regex");
        Self {
            inner: MatcherType::Raw(inner),
        }
    }

    /// Chain several matchers together in sequence.
    pub fn from_sequence(seq: Vec<MatchPart>) -> Self {
        Self {
            inner: MatcherType::Sequence(seq),
        }
    }

    /// Combine several matchers in a way that only one of them can match at a time.
    pub fn from_alternation(alts: Vec<Matcher>) -> Self {
        Self {
            inner: MatcherType::Alternation(alts),
        }
    }

    pub(crate) fn compile(self, capture_index: &mut usize) -> Hir {
        let index = *capture_index;
        *capture_index += 1;
        let hir = match self.inner {
            MatcherType::Raw(mut hir) => {
                compile_raw(&mut hir, capture_index);
                hir
            }
            MatcherType::Sequence(matchers) => {
                let mut hirs = vec![];
                for matcher in matchers {
                    match matcher {
                        MatchPart::Matcher(matcher) => {
                            hirs.push(matcher.compile(capture_index));
                        }
                        MatchPart::Regex(cow) => {
                            let hir = regex_syntax::parse(&cow).expect("Failed to parse regex");
                            assert_eq!(
                                hir.properties().explicit_captures_len(),
                                0,
                                "sscanf: MatcherComponent::Regex must not contain any capture groups"
                            );
                            hirs.push(hir);
                        }
                        MatchPart::Literal(cow) => {
                            let hir = Hir::literal(cow.as_bytes());
                            hirs.push(hir);
                        }
                    }
                }
                Hir::concat(hirs)
            }
            MatcherType::Alternation(matchers) => {
                let hirs = matchers
                    .into_iter()
                    .map(|m| m.compile(capture_index))
                    .collect();
                Hir::alternation(hirs)
            }
        };
        let capture = Capture {
            index: index as u32,
            name: None,
            sub: Box::new(hir),
        };
        Hir::capture(capture)
    }

    /// Convert a matcher to a regex string.
    ///
    /// Note that this is an expensive operation and should only be used for debugging or testing purposes.
    ///
    /// Note that the resulting regex might be different from a regex passed to [`Matcher::from_regex`] due to
    /// optimizations and transformations applied by the regex engine.
    pub fn to_regex(&self) -> String {
        let mut capture_index = 0;
        let hir = self.clone().compile(&mut capture_index);
        hir.to_string()
    }
}

/// One component of e.g. a format string when converting it to a [`Matcher`].
#[derive(Debug, Clone)]
pub enum MatchPart {
    /// An inner matcher for fields etc.
    Matcher(Matcher),
    /// A regex string that should be matched. Must not contain any capture groups.
    Regex(Cow<'static, str>),
    /// A literal string that should be matched exactly.
    Literal(Cow<'static, str>),
}

impl MatchPart {
    /// Convenience method to create a [`MatchPart::Regex`] from a `String`, `&str`, or `Cow<str>`.
    pub fn regex(s: impl Into<Cow<'static, str>>) -> Self {
        MatchPart::Regex(s.into())
    }
    /// Convenience method to create a [`MatchPart::Literal`] from a `String`, `&str`, or `Cow<str>`.
    pub fn literal(s: impl Into<Cow<'static, str>>) -> Self {
        MatchPart::Literal(s.into())
    }
}

impl From<Matcher> for MatchPart {
    fn from(matcher: Matcher) -> Self {
        MatchPart::Matcher(matcher)
    }
}

/// Represents the type of matcher
///
/// Note that we could combine the Hirs together using the Hir::concat etc. methods, but we will need to
/// reconstruct the entire hir from scratch anyway at the end to set the capture indices, so we might as well
/// keep them separate for now.
#[derive(Debug, Clone)]
enum MatcherType {
    Raw(Hir),
    Sequence(Vec<MatchPart>),
    Alternation(Vec<Matcher>),
}

fn compile_raw(hir: &mut Hir, capture_index: &mut usize) {
    if hir.properties().explicit_captures_len() == 0 {
        return; // No captures to process
    }
    let kind = std::mem::replace(hir, Hir::empty()).into_kind();
    use regex_syntax::hir::HirKind;
    match kind {
        HirKind::Capture(mut capture) => {
            capture.index = *capture_index as u32;
            *capture_index += 1;

            compile_raw(&mut capture.sub, capture_index);
            *hir = Hir::capture(capture);
        }

        HirKind::Repetition(mut repetition) => {
            compile_raw(&mut repetition.sub, capture_index);
            *hir = Hir::repetition(repetition);
        }
        HirKind::Concat(mut hirs) => {
            for sub_hir in &mut hirs {
                compile_raw(sub_hir, capture_index);
            }
            *hir = Hir::concat(hirs);
        }
        HirKind::Alternation(mut hirs) => {
            for sub_hir in &mut hirs {
                compile_raw(sub_hir, capture_index);
            }
            *hir = Hir::alternation(hirs);
        }
        _ => unreachable!(
            "sscanf: HirKind {:?} was not supposed to have any captures",
            kind
        ),
    }
}
