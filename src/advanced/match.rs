use crate::{FromScanf, advanced::FormatOptions};

use regex_automata::{Span, util::captures::Captures};

#[expect(unused_imports, reason = "for doc links")]
use crate::advanced::{MatchPart, Matcher};

mod alt;
mod context;
mod seq;
mod template;

pub use alt::*;
pub use seq::*;

pub(crate) use context::*;
pub(crate) use template::*;

/// A tree representation of regex capture group matches.
///
/// This type is the parameter to the [`FromScanf::from_match`] method.
///
/// Use [`text()`](Self::text) for the full matched string, and the `as_*` methods to access specific matcher views.
///
/// Convenience methods include [`parse()`](Self::parse) to parse this match tree into a value.
///
/// ## On Optional Capture Groups
///
/// This documentation refers to "optional capture groups" or "capture groups that did not match" (what the regex
/// crate calls "capture groups that did not participate in the match"). These refer to
/// capture groups that are not guaranteed to match text when the regex is applied to a string. This can happen
/// when the capture group is optional in the regex, like `(x)?`, or when it is part of an alternation, like
/// `(x)|y`. In both cases, it is possible for the overall regex to match a string without that capture group
/// actually capturing any text.
///
/// In this crate, these capture groups are referred to as "optional" and are represented by `Option<Match>` in the
/// return type of [`as_opt()`](Self::as_opt).  
/// Note that there is **no** automatic handling of `Option` types in either the `sscanf` macro or the `FromScanf`
/// derive!
///
/// #### Example of using optional capture groups to parse an enum:
/// ```
/// use sscanf::advanced::{Matcher, Match, FormatOptions};
/// # #[derive(Debug, PartialEq, Eq)]
/// enum MyType<'a> {
///     Digits(usize),
///     Letters(&'a str),
/// }
/// impl<'input> sscanf::FromScanf<'input> for MyType<'input> {
///     // matches either digits or letters, but not both
///     fn get_matcher(_: &FormatOptions) -> Matcher {
///         Matcher::Alt(vec![
///             Matcher::from_regex(r"\d+").unwrap(),
///             Matcher::from_regex(r"[a-zA-Z]+").unwrap(),
///         ])
///     }
///
///     fn from_match(matches: Match<'_, 'input>, _: &FormatOptions) -> Option<Self> {
///         let matches = matches.as_alt();
///         let text = matches.get().text();
///         if matches.matched_index() == 0 {
///             // The first alternative matched (\d+)
///             Some(Self::Digits(text.parse().ok()?))
///         } else {
///             // exactly one of the capture groups will match
///             assert_eq!(matches.matched_index(), 1);
///             // The second alternative matched ([a-zA-Z]+)
///             Some(Self::Letters(text))
///         }
///     }
/// }
///
/// let digits = sscanf::sscanf!("123", "{MyType}").unwrap();
/// assert_eq!(digits, MyType::Digits(123));
///
/// let letters = sscanf::sscanf!("abc", "{MyType}").unwrap();
/// assert_eq!(letters, MyType::Letters("abc"));
/// ```
///
/// Side note: The derive macro uses this mechanism for enums. If it doesn't work for your enum, implement this trait
/// explicitly using regex alternations for variants, each wrapped in a capture group to check which variant matched:
/// `(...)|(...)|(...)`.
///
/// Because of this, there are utility methods on [`Matcher`](crate::advanced::Matcher) for combining matchers:
/// ```
/// # use sscanf::advanced::Matcher;
/// # fn get_matcher() -> Matcher {
/// Matcher::Alt(vec![
///     Matcher::from_regex(r"\d+").unwrap(),
///     Matcher::from_regex(r"[a-zA-Z]+").unwrap(),
/// ])
/// # }
/// ```
///
/// ## Lifetime Parameters
/// The first lifetime parameter (`'t`) is the lifetime of the match tree itself. Match trees are only valid within
/// [`FromScanf::from_match`] and can't be stored. This can usually be set to `'_`.
///
/// The second lifetime parameter (`'input`) is the lifetime of the input string that was parsed to create this match
/// tree.  
/// If your type borrows parts of the input string, like `&str` does, you need to match the lifetime parameter on
/// your type to the `'input` parameter.
#[derive(Clone, Copy)]
pub struct Match<'t, 'input> {
    template: &'t MatchTreeTemplate,
    captures: &'t Captures,
    input: &'input str,
    full_text: &'input str,
    context: ContextChain<'t>,
}

impl<'t, 'input> Match<'t, 'input> {
    /// Internal constructor. `Match` can only be received as a parameter to `FromScanf::from_match` and from
    /// the methods on an existing `Match`.
    pub(crate) fn new(
        template: &'t MatchTreeTemplate,
        captures: &'t Captures,
        input: &'input str,
        current: Span,
        context: ContextChain<'t>,
    ) -> Self {
        Self {
            template,
            captures,
            input,
            full_text: &input[current],
            context,
        }
    }

    /// Returns the entire matched text for this match tree.
    pub fn text(&self) -> &'input str {
        self.full_text
    }

    /// Convenience method to call [`FromScanf::from_match`] with this match tree.
    ///
    /// The type `T` must implement the [`FromScanf`] trait, and this object must have been created from a match to
    /// [`T::get_matcher()`](FromScanf::get_matcher).
    pub fn parse<T: FromScanf<'input>>(&self, options: &FormatOptions) -> Option<T> {
        let context = self.context.and(Context::Parse(std::any::type_name::<T>()));
        T::from_match(Match { context, ..*self }, options)
    }

    /// Returns the match as a Vec of regex captures.
    ///
    /// Each entry in the Vec corresponds to a capture group in the regex. Note that they are each wrapped in an
    /// `Option`, because capture groups in optional sections and alternations might not have matched.
    ///
    /// ## Panics
    /// Panics if this `Match` was not created from a [`Matcher::Regex`].
    pub fn as_regex_matches(&'t self) -> Vec<Option<&'input str>> {
        let MatchTreeKind::Regex(range) = &self.template.kind else {
            panic!(
                "sscanf: Match::as_regex_matches called on a {}.\nContext: {}",
                self.template.kind_name(),
                self.context
            )
        };
        range
            .clone()
            .map(|i| self.captures.get_group(i).map(|span| &self.input[span]))
            .collect()
    }

    /// Returns the match as a [`SeqMatch`].
    ///
    /// ## Panics
    /// Panics if this `Match` was not created from a [`Matcher::Seq`].
    pub fn as_seq(&'t self) -> SeqMatch<'t, 'input> {
        let MatchTreeKind::Seq(children) = &self.template.kind else {
            panic!(
                "sscanf: Match::as_seq called on a {}.\nContext: {}",
                self.template.kind_name(),
                self.context,
            )
        };
        SeqMatch {
            children,
            captures: self.captures,
            input: self.input,
            full_text: self.full_text,
            context: self.context.and(Context::AsSeq),
        }
    }

    /// Returns the match as an [`AltMatch`].
    ///
    /// ## Panics
    /// Panics if this `Match` was not created from a [`Matcher::Alt`].
    pub fn as_alt(&'t self) -> AltMatch<'t, 'input> {
        let MatchTreeKind::Alt(children) = &self.template.kind else {
            panic!(
                "sscanf: Match::as_alt called on a {}.\nContext: {}",
                self.template.kind_name(),
                self.context,
            )
        };
        let Some((matched_index, child, span)) = children
            .iter()
            .enumerate()
            .find_map(|(i, child)| Some((i, child, self.captures.get_group(child.index)?)))
        else {
            panic!(
                "sscanf: AltMatch has no matching alternative!\nContext: {}",
                self.context,
            );
        };

        let child = Match::new(
            child,
            self.captures,
            self.input,
            span,
            self.context.and(Context::AsAlt(matched_index)),
        );
        AltMatch {
            matched_index,
            child,
            full_text: self.full_text,
        }
    }

    /// Returns the match as an [`AltMatch`], with enum variants as context.
    ///
    /// ## Panics
    /// Panics if this `Match` was not created from a [`Matcher::Alt`].
    pub fn as_alt_enum(&'t self, variants: &[&'static str]) -> AltMatch<'t, 'input> {
        let MatchTreeKind::Alt(children) = &self.template.kind else {
            panic!(
                "sscanf: Match::as_alt_enum called on a {}.\nContext: {}",
                self.template.kind_name(),
                self.context,
            )
        };

        assert_eq!(
            children.len(),
            variants.len(),
            "sscanf: Mismatch between number of alternatives and number of variant names provided to Match::as_alt_enum.\nContext: {}",
            self.context,
        );

        let Some((matched_index, child, span)) = children
            .iter()
            .enumerate()
            .find_map(|(i, child)| Some((i, child, self.captures.get_group(child.index)?)))
        else {
            panic!(
                "sscanf: AltMatch has no matching alternative!\nContext: {}",
                self.context,
            );
        };

        let child = Match::new(
            child,
            self.captures,
            self.input,
            span,
            self.context
                .and(Context::AsAltEnum(variants[matched_index])),
        );
        AltMatch {
            matched_index,
            child,
            full_text: self.full_text,
        }
    }

    /// Returns the match as an optional [`Match`]
    ///
    /// ## Panics
    /// Panics if this `Match` was not created from a [`Matcher::Optional`].
    pub fn as_opt(&'t self) -> Option<Match<'t, 'input>> {
        let MatchTreeKind::Optional(child) = &self.template.kind else {
            panic!(
                "sscanf: Match::as_opt called on a {}.\nContext: {}",
                self.template.kind_name(),
                self.context,
            )
        };
        let span = self.captures.get_group(child.index)?;
        Some(Match::new(
            child,
            self.captures,
            self.input,
            span,
            self.context.and(Context::AsOpt),
        ))
    }
}

impl std::fmt::Debug for Match<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.template.kind {
            MatchTreeKind::Regex(_) => {
                let captures = self.as_regex_matches();
                f.debug_struct("RegexMatch")
                    .field("full_text", &self.text())
                    .field("captures", &captures)
                    .finish()
            }
            MatchTreeKind::Seq(_) => self.as_seq().fmt(f),
            MatchTreeKind::Alt(_) => self.as_alt().fmt(f),
            MatchTreeKind::Optional(_) => {
                let opt = self.as_opt();
                f.debug_struct("OptionalMatch")
                    .field("full_text", &self.text())
                    .field("child", &opt)
                    .finish()
            }
        }
    }
}

impl std::fmt::Display for Match<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.text().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_panic_message_eq;

    #[test]
    fn has_correct_panics() {
        let parsed = regex_automata::meta::Regex::new("(a)(b)(c)(d)").unwrap();
        let captures = parsed.create_captures();

        let input = "abcd";

        let current = Span { start: 0, end: 4 };

        let context: ContextChain = Context::Root.into();

        let template_seq = MatchTreeTemplate {
            index: 0,
            kind: MatchTreeKind::Seq(vec![]),
        };
        let template_regex = MatchTreeTemplate {
            index: 0,
            kind: MatchTreeKind::Regex(1..2),
        };
        let template_alt = MatchTreeTemplate {
            index: 0,
            kind: MatchTreeKind::Alt(vec![]),
        };
        let template_opt = MatchTreeTemplate {
            index: 0,
            kind: MatchTreeKind::Optional(Box::new(MatchTreeTemplate {
                index: 1,
                kind: MatchTreeKind::Regex(2..3),
            })),
        };

        let match_tree_seq = Match::new(&template_seq, &captures, input, current, context);
        assert_panic_message_eq!(
            match_tree_seq.as_regex_matches(),
            r#"sscanf: Match::as_regex_matches called on a Sequence Match.
Context: sscanf"#
        );

        let context = context.and(Context::AsSeq);
        let match_tree_regex = Match::new(&template_regex, &captures, input, current, context);
        assert_panic_message_eq!(
            match_tree_regex.as_seq(),
            r#"sscanf: Match::as_seq called on a Regex Match.
Context: sscanf -> as_seq()"#
        );

        let context = context.and(Context::AsAltEnum("hi"));
        let match_tree_alt = Match::new(&template_alt, &captures, input, current, context);
        assert_panic_message_eq!(
            match_tree_alt.as_opt(),
            r#"sscanf: Match::as_opt called on a Alt Match.
Context: sscanf -> as_seq() -> as_alt(hi matched)"#
        );

        let context = context.and(Context::Parse("MyType"));
        let match_tree_opt = Match::new(&template_opt, &captures, input, current, context);
        assert_panic_message_eq!(
            match_tree_opt.as_alt(),
            r#"sscanf: Match::as_alt called on a Optional Match.
Context: sscanf -> as_seq() -> as_alt(hi matched) -> parse as MyType"#
        );
    }

    #[test]
    fn test_match_tree_debug() {
        let parsed =
            regex_automata::meta::Regex::new("([a-b]{2})((cd)|(ef))gh((i)?)((j)?)").unwrap();
        let mut captures = parsed.create_captures();

        let input = "abcdghj";

        parsed.captures(input, &mut captures);

        let current = Span {
            start: 0,
            end: input.len(),
        };

        let context: ContextChain = Context::Root.into();

        let template_regex = MatchTreeTemplate {
            index: 0,
            kind: MatchTreeKind::Seq(vec![
                Some(MatchTreeTemplate {
                    index: 1,
                    kind: MatchTreeKind::Regex(1..2),
                }),
                Some(MatchTreeTemplate {
                    index: 2,
                    kind: MatchTreeKind::Alt(vec![
                        MatchTreeTemplate {
                            index: 3,
                            kind: MatchTreeKind::Regex(3..4),
                        },
                        MatchTreeTemplate {
                            index: 4,
                            kind: MatchTreeKind::Regex(5..6),
                        },
                    ]),
                }),
                Some(MatchTreeTemplate {
                    index: 5,
                    kind: MatchTreeKind::Optional(Box::new(MatchTreeTemplate {
                        index: 6,
                        kind: MatchTreeKind::Regex(7..7),
                    })),
                }),
                Some(MatchTreeTemplate {
                    index: 7,
                    kind: MatchTreeKind::Optional(Box::new(MatchTreeTemplate {
                        index: 8,
                        kind: MatchTreeKind::Regex(9..9),
                    })),
                }),
            ]),
        };

        let match_tree_regex = Match::new(&template_regex, &captures, input, current, context);
        assert_eq!(
            format!("{:#?}", match_tree_regex),
            r#"Match::Seq {
    full_text: "abcdghj",
    children: [
        Some(
            RegexMatch {
                full_text: "ab",
                captures: [
                    Some(
                        "ab",
                    ),
                ],
            },
        ),
        Some(
            AltMatch {
                full_text: "cd",
                matched_index: 0,
                child: RegexMatch {
                    full_text: "cd",
                    captures: [
                        Some(
                            "cd",
                        ),
                    ],
                },
            },
        ),
        Some(
            OptionalMatch {
                full_text: "",
                child: None,
            },
        ),
        Some(
            OptionalMatch {
                full_text: "j",
                child: Some(
                    RegexMatch {
                        full_text: "j",
                        captures: [],
                    },
                ),
            },
        ),
    ],
    ..
}"#
        );
    }
}
