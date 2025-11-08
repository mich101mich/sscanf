use crate::{FromScanf, advanced::FormatOptions};

use regex_automata::{Span, util::captures::Captures};

#[allow(unused_imports)]
use crate::advanced::{MatchPart, Matcher}; // for links in docs

mod alt;
mod raw;
mod seq;
pub use alt::*;
pub use raw::*;
pub use seq::*;

/// Representation of the match of a capture group in a regex, arranged in a tree structure.
///
/// This type is the parameter to the [`FromScanf::from_match_tree`] method.
///
/// Use [`text()`](Self::text) to access the entire matched string, and one of the `as_*` methods to access the
/// individual matchers.
///
/// There are also convenience methods for parsing, like [`parse()`](Self::parse) for this match tree.
///
/// ## Guide to `panic!` vs `return None`
///
/// Assuming the following regex:
/// ```text
/// (\d+) item(s)?
/// ```
/// This regex has two capture groups, the first one is required, the second one is optional.
///
/// | Problem Description | Example | Action | Explanation |
/// |---------------------|---------|--------|-------------|
/// | The regex is too broad | The first capture group can match 100+ digits, but our final data type might not store that many | return&nbsp;`None` | This case should have been filtered by the regex, but wasn't. <br/>Note that this might be unavoidable. For example `u8`'s regex matches only three digits, but 999 is not a valid `u8` and has to be filtered during the parsing process |
/// | The `MatchTree` has fewer children than there are direct capture groups in the regex | The `MatchTree` only has 0 or 1 child | `panic!()` | This is a programming error in the calling code |
/// | You tried to access a capture group that does not exist | Attempting to access a third capture group | `panic!()` | This is a programming error in your code |
/// | An optional capture group did not match | the second group did not match an `s` | continue parsing | This is a valid case, so the parsing should be able to handle it. Otherwise, the group should be made non-optional |
/// | A non-optional capture group did not match | The first capture group is `None` | `panic!()` | This is a programming error in the calling code |
///
/// If a programming error occurs and you are certain that it is not your fault, please open an issue on GitHub.
///
/// ## Example structure
/// ```
/// # use sscanf::advanced::{Matcher, MatchTree, FormatOptions};
/// # struct MyType;
/// impl sscanf::FromScanf<'_> for MyType {
///     fn get_matcher(_: &FormatOptions) -> Matcher {
///         Matcher::from_regex(r"a(b)c(x)?d(ef(ghi)j(k))lm").unwrap()
///     }
///
///     fn from_match_tree(matches: MatchTree<'_, '_>, _: &FormatOptions) -> Option<Self> {
///         // This is what the complete match tree looks like:
///         // TODO: reimplement
///
/// //         assert_eq!(matches.text(), "abcdefghijklm");
/// //         assert_eq!(matches.num_children(), 3); // (b) (x) (ef..)
/// //
/// //         { // the "(b)" group
/// //             let b = matches.at(0);
/// //             assert_eq!(b.text(), "b");
/// //             assert_eq!(b.num_children(), 0); // no more capture groups within this group
/// //         }
/// //
/// //         { // the "(x)?" group (did not match)
/// //             let x = matches.get(1);
/// //             assert!(x.is_none());
/// //         }
/// //
/// //         { // the "(ef(ghi)j(k))" group
/// //             let efghijk = matches.at(2);
/// //             assert_eq!(efghijk.text(), "efghijk");
/// //             assert_eq!(efghijk.num_children(), 2); // (ghi) (k)
/// //
/// //             { // the "(ghi)" group
/// //                 let ghi = efghijk.at(0);
/// //                 assert_eq!(ghi.text(), "ghi");
/// //                 assert_eq!(ghi.num_children(), 0);
/// //             }
/// //
/// //             { // the "(k)" group
/// //                 let k = efghijk.at(1);
/// //                 assert_eq!(k.text(), "k");
/// //                 assert_eq!(k.num_children(), 0);
/// //             }
/// //         }
///
///         // ... do something with the matches ...
///         # Some(MyType)
///     }
/// }
/// sscanf::sscanf!("abcdefghijklm", "{MyType}").unwrap();
/// ```
///
/// ## On Optional Capture Groups
///
/// There are a lot of mentions of "optional capture groups" or "capture groups that did not match" (or as the regex
/// crate calls them: "capture groups that did not participate in the match") in this documentation. These refer to
/// capture groups that are not guaranteed to match text when the regex is applied to a string. This can happen
/// when the capture group is optional in the regex, like `(x)?`, or when it is part of an alternation, like
/// `(x)|y`. In both cases, it is possible for the overall regex to match a string without that capture group
/// actually capturing any text.
///
/// In this crate, these capture groups are referred to as "optional" and are represented by `Option<MatchTree>` in the
/// return type of [`as_opt()`](Self::as_opt).  
/// Note that there is **no** automatic handling of `Option` types in either the `sscanf` macro or the `FromScanf`
/// derive!
///
/// #### Example of using optional capture groups to parse an enum:
/// ```
/// use sscanf::advanced::{Matcher, MatchTree, FormatOptions};
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
///     fn from_match_tree(matches: MatchTree<'_, 'input>, _: &FormatOptions) -> Option<Self> {
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
/// Side note: This is the mechanism used by the derive macro when used on an enum. If the derive macro does not
/// work for your enum, consider implementing this trait in this exact way, using alternations in the regex for the
/// enum variants, each wrapped in a capture group to check which variant matched: `(...)|(...)|(...)`.
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
/// [`FromScanf::from_match_tree`] calls and can't be stored outside of that. This can usually be set to `'_`.
///
/// The second lifetime parameter (`'input`) is the lifetime of the input string that was parsed to create this match
/// tree.  
/// If your type borrows parts of the input string, like `&str` does, you need to match the lifetime parameter on
/// your type to the `'input` parameter.
#[derive(Clone, Copy)]
pub struct MatchTree<'t, 'input> {
    template: &'t MatchTreeTemplate,
    captures: &'t Captures,
    input: &'input str,
    full: &'input str,
    context: ContextChain<'t>,
}

impl<'t, 'input> MatchTree<'t, 'input> {
    /// Internal constructor. MatchTrees can only be received as a parameter to `FromScanf::from_match_tree` and from
    /// the methods on an existing `MatchTree`.
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
            full: &input[current],
            context,
        }
    }

    /// Returns the entire matched text for this match tree.
    pub fn text(&self) -> &'input str {
        self.full
    }

    /// Convenience method to call [`FromScanf::from_match_tree`] with this match tree.
    ///
    /// The type `T` must implement the [`FromScanf`] trait, and this object must have been created from a match to
    /// [`T::get_matcher()`](FromScanf::get_matcher).
    pub fn parse<T: FromScanf<'input>>(&self, format: &FormatOptions) -> Option<T> {
        let context = self.context.and(Context::Parse(std::any::type_name::<T>()));
        T::from_match_tree(MatchTree { context, ..*self }, format)
    }

    /// Returns the match as a [`RawMatch`].
    ///
    /// ## Panics
    /// Panics if this `MatchTree` was not created from a [`Matcher::Raw`].
    pub fn as_raw(&'t self) -> RawMatch<'t, 'input> {
        let MatchTreeKind::Raw(range) = &self.template.kind else {
            panic!(
                "sscanf: MatchTree::as_raw called on a {}.\nContext: {}",
                self.template.kind_name(),
                self.context
            )
        };
        RawMatch {
            indices: range.clone(),
            captures: self.captures,
            input: self.input,
            full: self.full,
            context: self.context.and(Context::AsRaw),
        }
    }

    /// Returns the match as a [`SeqMatch`].
    ///
    /// ## Panics
    /// Panics if this `MatchTree` was not created from a [`Matcher::Seq`].
    pub fn as_seq(&'t self) -> SeqMatch<'t, 'input> {
        let MatchTreeKind::Seq(children) = &self.template.kind else {
            panic!(
                "sscanf: MatchTree::as_seq called on a {}.\nContext: {}",
                self.template.kind_name(),
                self.context,
            )
        };
        SeqMatch {
            children,
            captures: self.captures,
            input: self.input,
            full: self.full,
            context: self.context.and(Context::AsSeq),
        }
    }

    /// Returns the match as an [`AltMatch`].
    ///
    /// ## Panics
    /// Panics if this `MatchTree` was not created from a [`Matcher::Alt`].
    pub fn as_alt(&'t self) -> AltMatch<'t, 'input> {
        let MatchTreeKind::Alt(children) = &self.template.kind else {
            panic!(
                "sscanf: MatchTree::as_alt called on a {}.\nContext: {}",
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

        let child = MatchTree::new(
            child,
            self.captures,
            self.input,
            span,
            self.context.and(Context::AltMatch(matched_index)),
        );
        AltMatch {
            matched_index,
            child,
            full: self.full,
        }
    }

    /// Returns the match as an optional [`MatchTree`]
    ///
    /// ## Panics
    /// Panics if this `MatchTree` was not created from a [`Matcher::Optional`].
    pub fn as_opt(&'t self) -> Option<MatchTree<'t, 'input>> {
        let MatchTreeKind::Opt(child) = &self.template.kind else {
            panic!(
                "sscanf: MatchTree::as_opt called on a {}.\nContext: {}",
                self.template.kind_name(),
                self.context,
            )
        };
        let span = self.captures.get_group(child.index)?;
        Some(MatchTree::new(
            child,
            self.captures,
            self.input,
            span,
            self.context.and(Context::AsOpt),
        ))
    }
}

impl std::fmt::Debug for MatchTree<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.template.kind {
            MatchTreeKind::Raw(_) => self.as_raw().fmt(f),
            MatchTreeKind::Seq(_) => self.as_seq().fmt(f),
            MatchTreeKind::Alt(_) => self.as_alt().fmt(f),
            MatchTreeKind::Opt(_) => self.as_opt().fmt(f),
        }
    }
}

impl std::fmt::Display for MatchTree<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.text().fmt(f)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Context {
    // MatchTree methods
    Parse(&'static str),
    AsRaw,
    AsSeq,
    AltMatch(usize),
    AsOpt,

    // SeqMatch methods
    At(usize),
    Get(usize),
    ParseAt(&'static str, usize),
    ParseField(&'static str, usize, &'static str),

    // other
    Named(&'static str),
}

#[derive(Clone, Copy)]
pub(crate) struct ContextChain<'t> {
    current: Context,
    parent: Option<&'t ContextChain<'t>>,
}
impl<'t> ContextChain<'t> {
    fn and(&'t self, child: Context) -> Self {
        Self {
            current: child,
            parent: Some(self),
        }
    }
}
impl From<Context> for ContextChain<'_> {
    fn from(context: Context) -> Self {
        Self {
            current: context,
            parent: None,
        }
    }
}
impl std::fmt::Display for ContextChain<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parent) = &self.parent {
            parent.fmt(f)?;
            f.write_str(" -> ")?;
        }
        match &self.current {
            Context::Parse(ty) => write!(f, "parse::<{ty}>()"),
            Context::AsRaw => f.write_str("as_raw()"),
            Context::AsSeq => f.write_str("as_seq()"),
            Context::AltMatch(index) => write!(f, "Alt ({index} matched)"),
            Context::AsOpt => f.write_str("as_opt()"),

            Context::At(index) => write!(f, "at({index})"),
            Context::Get(index) => write!(f, "get({index})"),
            Context::ParseAt(ty, index) => write!(f, "parse {index} as {ty}"),
            Context::ParseField(name, index, ty) => {
                write!(f, "parse .{name} (parse {index} as {ty})")
            }

            Context::Named(name) => f.write_str(name),
        }
    }
}

/// The source structure of a MatchTree, consisting of only the indices in the capture group list.
#[derive(Debug)]
pub(crate) struct MatchTreeTemplate {
    pub index: usize,
    pub kind: MatchTreeKind,
}

#[derive(Debug)]
pub(crate) enum MatchTreeKind {
    Raw(std::ops::Range<usize>),
    Seq(Vec<Option<MatchTreeTemplate>>),
    Alt(Vec<MatchTreeTemplate>),
    Opt(Box<MatchTreeTemplate>),
}

impl MatchTreeTemplate {
    pub fn kind_name(&self) -> &'static str {
        match &self.kind {
            MatchTreeKind::Raw(_) => "RawMatch",
            MatchTreeKind::Seq(_) => "SeqMatch",
            MatchTreeKind::Alt(_) => "AltMatch",
            MatchTreeKind::Opt(_) => "Optional Match",
        }
    }
}
