#![allow(
    unused,
    reason = "These are general utilities that I copy-paste between projects"
)]

use crate::*;

mod visitors {
    pub mod lifetime;
}
pub use visitors::lifetime::*;

/// A workaround for Spans on stable Rust.
///
/// Span manipulation doesn't work on stable Rust, which also means that spans cannot be joined
/// together. This means that any compiler errors that occur would only point at the first token
/// of the spanned expression, which is not very helpful.
///
/// The workaround, as demonstrated by `Error::new_spanned`, is to have the first part of the
/// spanned expression be spanned with the first part of the source span, and the second part of the
/// spanned expression be spanned with the second part of the source span. The compiler only looks
/// at the start and end of the span and underlines everything in between, so this works.
#[derive(Copy, Clone)]
pub struct FullSpan(Span, Span);

impl FullSpan {
    pub fn from_span(span: Span) -> Self {
        Self(span, span)
    }
    pub fn from_spanned<T: ToTokens>(span: &T) -> Self {
        let mut tokens = span.to_token_stream().into_iter().map(|t| t.span());
        let start = tokens.next().unwrap_or(Span::call_site());
        let end = tokens.last().unwrap_or(start);
        Self(start, end)
    }
    pub fn apply(self, a: TokenStream, b: TokenStream) -> TokenStream {
        let mut ret = self.apply_start(a);
        ret.extend(self.apply_end(b));
        ret
    }
    pub fn apply_start(self, a: TokenStream) -> TokenStream {
        a.with_span(self.0)
    }
    pub fn apply_end(self, b: TokenStream) -> TokenStream {
        b.with_span(self.1)
    }
}

/// Find the closest match to a string in a list of strings.
pub fn find_closest<'a>(s: &str, compare: &[&'a str]) -> Option<&'a str> {
    let mut best_confidence = 0.8; // minimum confidence
    let mut best_match = None;
    for valid in compare {
        let confidence = strsim::jaro_winkler(s, valid);
        if confidence > best_confidence {
            best_confidence = confidence;
            best_match = Some(*valid);
        }
    }
    best_match
}
/// Find the closest match to a string in a list of elements, removing it.
pub fn take_closest<T: Display>(s: &str, compare: &mut Vec<T>) -> Option<T> {
    let mut best_confidence = 0.8; // minimum confidence
    let mut best_index = None;
    for (i, valid) in compare.iter().enumerate() {
        let confidence = strsim::jaro_winkler(s, &valid.to_string());
        if confidence > best_confidence {
            best_confidence = confidence;
            best_index = Some(i);
        }
    }
    best_index.map(|index| compare.remove(index))
}

/// Format a list of items as a comma-separated list, with "or" before the last item.
pub fn list_items<T: Display>(items: &[T]) -> String {
    list_items_with(items, |x| x)
}

/// Format a list of items as a comma-separated list, with "or" before the last item.
pub fn list_items_quoted<T: Display>(items: &[T], quote: char) -> String {
    list_items_with(items, |x| format!("{quote}{x}{quote}"))
}

/// Format a list of items as a comma-separated list, with "or" before the last item.
pub fn list_items_with<'a, T, D: Display + 'a>(
    items: &'a [T],
    mut display: impl FnMut(&'a T) -> D,
) -> String {
    match items {
        [] => String::new(),
        [x] => display(x).to_string(),
        [a, b] => format!("{} or {}", display(a), display(b)),
        [start @ .., last] => {
            use std::fmt::Write;
            let mut s = String::new();
            for item in start {
                write!(s, "{}, ", display(item)).unwrap();
            }
            write!(s, "or {}", display(last)).unwrap();
            s
        }
    }
}

/// Extension trait for [`TokenStream`] that allows setting the span of all tokens in the stream.
pub trait TokenStreamExt {
    fn set_span(&mut self, span: Span);
    fn with_span(self, span: Span) -> Self;
}
impl TokenStreamExt for TokenStream {
    fn set_span(&mut self, span: Span) {
        let old = std::mem::replace(self, TokenStream::new());
        *self = old.with_span(span);
    }
    fn with_span(self, span: Span) -> Self {
        self.into_iter()
            .map(|mut t| {
                if let proc_macro2::TokenTree::Group(ref mut g) = t {
                    *g = proc_macro2::Group::new(g.delimiter(), g.stream().with_span(span));
                }
                t.set_span(span);
                t
            })
            .collect()
    }
}

// WORKAROUNDS: proc_macro(1) has stabilized several methods on Span, but proc_macro2 has not.
// Will be removed once proc_macro2 stabilizes these methods.

pub trait SpanExt {
    fn stable_start(&self) -> Span;
    fn stable_end(&self) -> Span;
    fn stable_line(&self) -> usize;
    fn stable_column(&self) -> usize;
    fn stable_file(&self) -> String;
}
#[cfg(not(test))]
impl SpanExt for Span {
    fn stable_start(&self) -> Span {
        self.unwrap().start().into() // Span2 -> Span1 -> call start() -> Span2
    }
    fn stable_end(&self) -> Span {
        self.unwrap().end().into() // Span2 -> Span1 -> call end() -> Span2
    }
    fn stable_line(&self) -> usize {
        self.unwrap().line() // Span2 -> Span1 -> line()
    }
    fn stable_column(&self) -> usize {
        self.unwrap().column() // Span2 -> Span1 -> column()
    }
    fn stable_file(&self) -> String {
        self.unwrap().file()
    }
}
#[cfg(test)]
impl SpanExt for Span {
    fn stable_start(&self) -> Span {
        // Note: Span::unwrap() returns a proc_macro(1) Span, which is not available outside of procedural macros, aka when testing.
        *self
    }
    fn stable_end(&self) -> Span {
        *self
    }
    fn stable_line(&self) -> usize {
        999
    }
    fn stable_column(&self) -> usize {
        123
    }
    fn stable_file(&self) -> String {
        "/inside/a/test.rs".to_string()
    }
}

pub trait ToTokensExt {
    fn start_span(&self) -> Span;
    fn end_span(&self) -> Span;
}

impl<T: ToTokens> ToTokensExt for T {
    fn start_span(&self) -> Span {
        self.to_token_stream()
            .into_iter()
            .next()
            .map_or_else(Span::call_site, |t| t.span().stable_start())
    }
    fn end_span(&self) -> Span {
        self.to_token_stream()
            .into_iter()
            .last()
            .map_or_else(Span::call_site, |t| t.span().stable_end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Utility macro for other tests: Asserts that the given block or statement throws a panic with the given message.
    #[macro_export]
    macro_rules! assert_panic_message_eq {
        ( $block:block, $message:literal $(,)? ) => {
            let Err(error) = std::panic::catch_unwind(move || $block) else {
                panic!("code {} did not panic", stringify!($block));
            };
            if let Some(s) = error.downcast_ref::<&'static str>() {
                assert_eq!(*s, $message);
            } else if let Some(s) = error.downcast_ref::<String>() {
                assert_eq!(s, $message);
            } else {
                panic!("unexpected panic payload: {:?}", error);
            }
        };
        ( $expression:expr, $message:literal $(,)? ) => {
            assert_panic_message_eq!(
                {
                    $expression; // avoid problems with lifetimes by not returning the value
                },
                $message
            );
        };
        ( $statement:stmt, $message:literal $(,)? ) => {
            assert_panic_message_eq!({ $statement }, $message);
        };
    }

    #[test]
    fn find_closest_basic() {
        let options = ["apple", "banana", "cherry", "date"];

        assert_eq!(find_closest("appl", &options), Some("apple"));
        assert_eq!(find_closest("bannana", &options), Some("banana"));
        assert_eq!(find_closest("cheri", &options), Some("cherry"));
        assert_eq!(find_closest("dat", &options), Some("date"));

        assert_eq!(find_closest("xyz", &options), None);
    }

    #[test]
    fn take_closest_basic() {
        let mut options = vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
            "date".to_string(),
        ];

        assert_eq!(
            take_closest("appl", &mut options),
            Some("apple".to_string())
        );
        assert_eq!(take_closest("appl", &mut options), None); // already taken

        assert_eq!(
            take_closest("bannana", &mut options),
            Some("banana".to_string())
        );
        assert_eq!(
            take_closest("cheri", &mut options),
            Some("cherry".to_string())
        );
        assert_eq!(take_closest("dat", &mut options), Some("date".to_string()));

        assert_eq!(take_closest("xyz", &mut options), None);
    }

    #[test]
    fn list_items_empty() {
        let items: [&str; 0] = [];
        assert_eq!(list_items(&items), "");
        assert_eq!(list_items_quoted(&items, '"'), "");
        assert_eq!(list_items_with(&items, |x| format!("{x} ^ {x}")), "");
    }

    #[test]
    fn list_items_single() {
        let items = ["apple"];
        assert_eq!(list_items(&items), "apple");
        assert_eq!(list_items_quoted(&items, '`'), "`apple`");
        assert_eq!(
            list_items_with(&items, |x| format!("{x} ^ {x}")),
            "apple ^ apple"
        );
    }

    #[test]
    fn list_items_two() {
        let items = ["apple", "banana"];
        assert_eq!(list_items(&items), "apple or banana");
        assert_eq!(list_items_quoted(&items, '\''), "'apple' or 'banana'");
        assert_eq!(
            list_items_with(&items, |x| format!("{x} ^ {x}")),
            "apple ^ apple or banana ^ banana"
        );
    }

    #[test]
    fn list_items_many() {
        let items = ["apple", "banana", "cherry"];
        assert_eq!(list_items(&items), "apple, banana, or cherry");
        assert_eq!(
            list_items_quoted(&items, '"'),
            "\"apple\", \"banana\", or \"cherry\""
        );
        assert_eq!(
            list_items_with(&items, |x| format!("{x} ^ {x}")),
            "apple ^ apple, banana ^ banana, or cherry ^ cherry"
        );

        let items = ["apple", "banana", "cherry", "date"];
        assert_eq!(list_items(&items), "apple, banana, cherry, or date");
        assert_eq!(
            list_items_quoted(&items, '`'),
            "`apple`, `banana`, `cherry`, or `date`"
        );
        assert_eq!(
            list_items_with(&items, |x| format!("{x} ^ {x}")),
            "apple ^ apple, banana ^ banana, cherry ^ cherry, or date ^ date"
        );
    }
}
