use super::{MatchTreeKind, MatchTreeTemplate};

use std::borrow::Cow;

use regex_syntax::hir::{Capture, Hir};

/// TODO:
#[derive(Debug, Clone)]
pub enum Matcher {
    /// A raw regex matcher. Created using [`Matcher::from_regex`].
    Regex(RegexMatcher),
    /// Chain several parts together in sequence.
    Seq(Vec<MatchPart>),
    /// Combine several matchers in a way that only one of them can match at a time.
    Alt(Vec<Matcher>),
    /// An optional matcher.
    Optional(Box<Matcher>),
}

/// Implementation details of [`Matcher::Regex`].
#[derive(Debug, Clone)]
pub struct RegexMatcher {
    hir: Hir,
}

impl Matcher {
    /// Create a new matcher from a regex string.
    pub fn from_regex(regex: &str) -> Result<Self, String> {
        regex_syntax::parse(regex)
            .map(|hir| Self::Regex(RegexMatcher { hir }))
            .map_err(|err| err.to_string())
    }

    /// Returns a new matcher that makes this matcher optional.
    pub fn optional(self) -> Self {
        Matcher::Optional(Box::new(self))
    }

    /// Internal constructor for a matcher from a raw HIR. Not public to avoid having a dependency in the public API.
    pub(crate) fn from_raw(hir: Hir) -> Self {
        Self::Regex(RegexMatcher { hir })
    }

    pub(crate) fn compile(
        self,
        capture_index: &mut usize,
    ) -> Result<(Hir, MatchTreeTemplate), String> {
        let index = *capture_index;
        *capture_index += 1;
        let (hir, kind) = match self {
            Matcher::Regex(RegexMatcher { mut hir }) => {
                let start_index = *capture_index;
                compile_raw(&mut hir, capture_index);
                let end_index = *capture_index;
                (hir, MatchTreeKind::Regex(start_index..end_index))
            }
            Matcher::Seq(matchers) => {
                let mut hirs = vec![];
                let mut children = vec![];
                for matcher in matchers {
                    match matcher {
                        MatchPart::Matcher(matcher) => {
                            let (hir, child_index) = matcher.compile(capture_index)?;
                            hirs.push(hir);
                            children.push(Some(child_index));
                        }
                        MatchPart::Regex(cow) => {
                            let hir = regex_syntax::parse(&cow)
                                .map_err(|err| format!("sscanf: Invalid regex segment: {err}"))?;
                            assert_eq!(
                                hir.properties().explicit_captures_len(),
                                0,
                                "sscanf: MatchPart::Regex must not contain any capture groups"
                            );
                            hirs.push(hir);
                            children.push(None);
                        }
                        MatchPart::Literal(Cow::Owned(s)) => {
                            let hir = Hir::literal(s.into_bytes().into_boxed_slice());
                            hirs.push(hir);
                            children.push(None);
                        }
                        MatchPart::Literal(Cow::Borrowed(s)) => {
                            let hir = Hir::literal(s.as_bytes());
                            hirs.push(hir);
                            children.push(None);
                        }
                    }
                }
                (Hir::concat(hirs), MatchTreeKind::Seq(children))
            }
            Matcher::Alt(matchers) => {
                let (hirs, children) = matchers
                    .into_iter()
                    .map(|m| m.compile(capture_index))
                    .collect::<Result<(Vec<_>, Vec<_>), _>>()?;
                (Hir::alternation(hirs), MatchTreeKind::Alt(children))
            }
            Matcher::Optional(matcher) => {
                let (hir, child_index) = matcher.compile(capture_index)?;
                let hir = Hir::repetition(regex_syntax::hir::Repetition {
                    min: 0,
                    max: Some(1),
                    greedy: true,
                    sub: Box::new(hir),
                });
                (hir, MatchTreeKind::Optional(Box::new(child_index)))
            }
        };
        let capture = Capture {
            index: index as u32,
            name: None,
            sub: Box::new(hir),
        };
        Ok((Hir::capture(capture), MatchTreeTemplate { index, kind }))
    }

    /// Convert a matcher to a regex string.
    ///
    /// Note that this is an expensive operation and should only be used for debugging or testing purposes.
    ///
    /// Note that the resulting regex might be different from a regex passed to [`Matcher::from_regex`] due to
    /// optimizations and transformations applied by the regex engine.
    #[track_caller]
    pub fn to_regex(&self) -> String {
        let mut capture_index = 0;
        let hir = self.clone().compile(&mut capture_index).unwrap();
        hir.0.to_string()
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
    /// Convenience method to create a [`MatchPart::Regex`] from a `String`, `&'static str`, or `Cow<str>`.
    pub fn regex(s: impl Into<Cow<'static, str>>) -> Self {
        MatchPart::Regex(s.into())
    }
    /// Convenience method to create a [`MatchPart::Literal`] from a `String`, `&'static str`, or `Cow<str>`.
    pub fn literal(s: impl Into<Cow<'static, str>>) -> Self {
        MatchPart::Literal(s.into())
    }
}

impl From<Matcher> for MatchPart {
    fn from(matcher: Matcher) -> Self {
        MatchPart::Matcher(matcher)
    }
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
