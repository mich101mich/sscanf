use crate::FromScanf;

use regex_automata::{Span, util::captures::Captures};

/// Representation of the match of a capture group in a regex, arranged in a tree structure.
///
/// This type is the parameter to the [`FromScanf::from_match_tree`] method.
///
/// Use [`text()`](Self::text) to access the entire matched string, and [`get()`](Self::get)/[`at()`](Self::at)
/// to access matches to inner capture groups.
///
/// There are also convenience methods for parsing, like [`parse()`](Self::parse) for this match tree, and
/// [`parse_at()`](Self::parse_at) for inner matches.
///
/// Note that a good amount of effort is spent on providing panic messages. Since the regex used to parse the input
/// is a compile-time constant, accessing capture groups that do not exist or that should not be optional is a
/// programming error. The `Option` returned by [`FromScanf::from_match_tree`] is really only `None` if the regex of
/// the type can't be made specific enough to exactly filter input if and only if it can be converted to the type.
///
/// To still provide debugging options for manual implementations, the `MatchTree` provides some context as to where
/// the error occurred in the parsing tree. This can only be done if the parsing structure is mostly handled by the
/// `MatchTree` by e.g. calling [`parse_at()`](Self::parse_at) rather than [`get()`](Self::get) with a manual unwrap
/// and parse.
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
/// | The regex is too broad | The first capture group can match 100+ digits, but our final data type might not store that many | return&nbsp;`None` | This case should have been filtered by the regex, but wasn't. <br/>Note that this might be unavoidable. For example `u8::REGEX` matches only three digits, but 999 is not a valid `u8` and has to be filtered during the parsing process |
/// | The `MatchTree` has fewer children than there are direct capture groups in the regex | The `MatchTree` only has 0 or 1 child | `panic!()` | This is a programming error in the calling code |
/// | You tried to access a capture group that does not exist | Attempting to access a third capture group | `panic!()` | This is a programming error in your code |
/// | An optional capture group did not match | the second group did not match an `s` | continue parsing | This is a valid case, so the parsing should be able to handle it. Otherwise, the group should be made non-optional |
/// | A non-optional capture group did not match | The first capture group is `None` | `panic!()` | This is a programming error in the calling code |
///
/// If a programming error occurs and you are certain that it is not your fault, please open an issue on GitHub.
///
/// ## Example structure
/// ```
/// # use sscanf::MatchTree;
/// # struct MyType;
/// impl sscanf::FromScanf<'_> for MyType {
///     const REGEX: &'static str = "a(b)c(x)?d(ef(ghi)j(k))lm";
///
///     fn from_match(_: &str) -> Option<Self> { None }
///
///     fn from_match_tree(matches: MatchTree<'_, '_>) -> Option<Self> {
///         // This is what the complete match tree looks like:
///
///         assert_eq!(matches.text(), "abcdefghijklm");
///         assert_eq!(matches.num_children(), 3); // (b) (x) (ef..)
///
///         { // the "(b)" group
///             let b = matches.at(0);
///             assert_eq!(b.text(), "b");
///             assert_eq!(b.num_children(), 0); // no more capture groups within this group
///         }
///
///         { // the "(x)?" group (did not match)
///             let x = matches.get(1);
///             assert!(x.is_none());
///         }
///
///         { // the "(ef(ghi)j(k))" group
///             let efghijk = matches.at(2);
///             assert_eq!(efghijk.text(), "efghijk");
///             assert_eq!(efghijk.num_children(), 2); // (ghi) (k)
///
///             { // the "(ghi)" group
///                 let ghi = efghijk.at(0);
///                 assert_eq!(ghi.text(), "ghi");
///                 assert_eq!(ghi.num_children(), 0); // no more capture groups within this group
///             }
///
///             { // the "(k)" group
///                 let k = efghijk.at(1);
///                 assert_eq!(k.text(), "k");
///                 assert_eq!(k.num_children(), 0); // no more capture groups within this group
///             }
///         }
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
/// return type of [`get()`](Self::get).  
/// Note that there is **no** automatic handling of `Option` types in either the `sscanf` macro or the `FromScanf`
/// derive!
///
/// #### Example of using optional capture groups to parse an enum:
/// ```
/// # #[derive(Debug, PartialEq, Eq)]
/// enum MyType<'a> {
///     Digits(&'a str),
///     Letters(&'a str),
/// }
/// impl<'input> sscanf::FromScanf<'input> for MyType<'input> {
///     // matches either digits or letters, but not both
///     const REGEX: &'static str = r"(\d+)|([a-zA-Z]+)";
///
///     fn from_match(_: &str) -> Option<Self> { None }
///
///     fn from_match_tree(matches: sscanf::MatchTree<'_, 'input>) -> Option<Self> {
///         if let Some(digits) = matches.get(0) {
///             assert!(matches.get(1).is_none()); // only one of the capture groups matches
///             Some(Self::Digits(digits.text()))
///         } else {
///             // exactly one of the capture groups will match
///             let letters = matches.at(1);
///             Some(Self::Letters(letters.text()))
///         }
///     }
/// }
///
/// let digits = sscanf::sscanf!("123", "{MyType}").unwrap();
/// assert_eq!(digits, MyType::Digits("123"));
///
/// let letters = sscanf::sscanf!("abc", "{MyType}").unwrap();
/// assert_eq!(letters, MyType::Letters("abc"));
/// ```
///
/// Side note: This is the mechanism used by the derive macro when used on an enum. If the derive macro does not
/// work for your enum, consider implementing this trait in this exact way, using alternations in the regex for the
/// enum variants, each wrapped in a capture group to check which variant matched: `(...)|(...)|(...)`.
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
    inner: &'t MatchTreeIndex,
    captures: &'t Captures,
    input: &'input str,
    full: &'input str,
    context: ContextChain<'t>,
}

impl<'t, 'input> MatchTree<'t, 'input> {
    /// Internal constructor. MatchTrees can only be received as a parameter to `FromScanf::from_match_tree` and from
    /// the methods on an existing `MatchTree`.
    pub(crate) fn new(
        inner: &'t MatchTreeIndex,
        captures: &'t Captures,
        input: &'input str,
        current: Span,
        context: ContextChain<'t>,
    ) -> Self {
        Self {
            inner,
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

    /// Returns the number of children in this match tree, i.e. the number of inner capture groups that exist in the
    /// regex.
    pub fn num_children(&self) -> usize {
        self.inner.children.len()
    }

    /// Convenience method to call [`FromScanf::from_match_tree`] with this match tree.
    ///
    /// The type `T` must implement the `FromScanf` trait, and this object must have been created from a match to
    /// `T::REGEX`.
    pub fn parse<T: FromScanf<'input>>(&self) -> Option<T> {
        let context = self.context.and(Context::Parse(std::any::type_name::<T>()));
        T::from_match_tree(MatchTree { context, ..*self })
    }

    /// Directly parse one of the inner matches at the given index, if it participated in the match.
    ///
    /// Do not use this method for optional capture groups, as it will panic if the match is `None`. Use
    /// [`get()`](Self::get) for that and handle the `None` case yourself.
    ///
    /// Shorthand for `self.at(index).parse()`. The same restrictions apply as for [`parse()`](Self::parse).
    #[track_caller]
    pub fn parse_at<T: FromScanf<'input>>(&self, index: usize) -> Option<T> {
        let context = Context::ParseAt(std::any::type_name::<T>(), index);
        T::from_match_tree(self.inner_at(index, context))
    }

    /// Internal method to parse a field at the given index, asserting that it exists.
    #[doc(hidden)]
    #[track_caller]
    pub fn parse_field<T: FromScanf<'input>>(&self, name: &'static str, index: usize) -> Option<T> {
        let context = Context::ParseField(name, index, std::any::type_name::<T>());
        T::from_match_tree(self.inner_at(index, context))
    }

    /// Returns the inner match at the given index, asserting that it exists.
    ///
    /// Don't use this method for optional capture groups, as it will panic if the match is `None`. Use
    /// [`get()`](Self::get) for that.
    ///
    /// ## Panics
    /// Panics if the index is out of bounds or if the inner match at that index is `None`.
    #[track_caller]
    pub fn at(&'t self, index: usize) -> MatchTree<'t, 'input> {
        self.inner_at(index, Context::At(index))
    }

    /// Returns the inner match at the given index, if it participated in the match.
    ///
    /// Use this method for optional capture groups. If the capture group is non-optional, prefer using
    /// [`at()`](Self::at) instead for a more descriptive panic message.
    ///
    /// ## Panics
    /// Panics if the index is out of bounds.
    #[track_caller]
    pub fn get(&'t self, index: usize) -> Option<MatchTree<'t, 'input>> {
        self.inner_get(index, Context::Get(index))
    }

    #[track_caller]
    fn inner_get(&'t self, index: usize, context: Context) -> Option<MatchTree<'t, 'input>> {
        let context = self.context.and(context);
        let Some(child) = self.inner.children.get(index) else {
            panic!(
                "sscanf: index {index} out of bounds in MatchTree with {} children. Is there a custom regex with an incorrect number of capture groups?\nContext: {}",
                self.inner.children.len(),
                context.print()
            );
        };
        self.captures
            .get_group(child.index)
            .map(|span| MatchTree::new(child, self.captures, self.input, span, context))
    }

    #[track_caller]
    fn inner_at(&'t self, index: usize, context: Context) -> MatchTree<'t, 'input> {
        match self.inner_get(index, context) {
            Some(child) => child,
            None => {
                panic!(
                    "sscanf: inner match at index {index} is None. Are there any unescaped `?` or `|` in a regex?\nContext: {}",
                    self.context.and(context).print()
                );
            }
        }
    }
}

impl std::fmt::Debug for MatchTree<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MatchTree")
            .field("text", &self.text())
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for MatchTree<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.text().fmt(f)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Context {
    At(usize),
    Get(usize),
    Parse(&'static str),
    ParseAt(&'static str, usize),
    Named(&'static str),
    ParseField(&'static str, usize, &'static str),
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
    fn print(&self) -> String {
        let mut out = String::new();
        self.append_to(&mut out);
        out
    }
    fn append_to(&self, out: &mut String) {
        if let Some(parent) = &self.parent {
            parent.append_to(out);
            out.push_str(" -> ");
        }
        use std::fmt::Write;
        match &self.current {
            Context::At(index) => write!(out, "assert group {index}").unwrap(),
            Context::Get(index) => write!(out, "get group {index}").unwrap(),
            Context::Parse(ty) => write!(out, "parse as {ty}").unwrap(),
            Context::ParseAt(ty, index) => write!(out, "parse group {index} as {ty}").unwrap(),
            Context::Named(name) => out.push_str(name),
            Context::ParseField(name, index, ty) => {
                write!(out, "parse field .{name} from group {index} as {ty}").unwrap()
            }
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

/// The source structure of a MatchTree, consisting of only the indices in the capture group list.
pub(crate) struct MatchTreeIndex {
    pub index: usize,
    pub children: Vec<MatchTreeIndex>,
}

pub(crate) fn fill_index(hir: &regex_syntax::hir::Hir, out: &mut MatchTreeIndex) {
    match hir.kind() {
        regex_syntax::hir::HirKind::Capture(capture) => {
            let mut child = MatchTreeIndex {
                index: capture.index as usize,
                children: Vec::new(),
            };
            fill_index(&capture.sub, &mut child);
            out.children.push(child);
        }

        regex_syntax::hir::HirKind::Repetition(repetition) => {
            fill_index(&repetition.sub, out);
        }
        regex_syntax::hir::HirKind::Concat(hirs) => {
            for sub_hir in hirs {
                fill_index(sub_hir, out);
            }
        }
        regex_syntax::hir::HirKind::Alternation(hirs) => {
            for sub_hir in hirs {
                fill_index(sub_hir, out);
            }
        }
        _ => {}
    }
}
