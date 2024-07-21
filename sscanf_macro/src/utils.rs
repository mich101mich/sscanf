#![allow(unused)]

use crate::*;

/// A workaround for Spans on stable Rust.
///
/// Span manipulation doesn't work on stable Rust, which also means that spans cannot be joined
/// together. This means that any compiler errors that occur would only point at the first token
/// of the spanned expression, which is not very helpful.
///
/// The workaround, as demonstrated by `syn::Error::new_spanned`, is to have the first part of the
/// spanned expression be spanned with the first part of the source span, and the second part of the
/// spanned expression be spanned with the second part of the source span. The compiler only looks
/// at the start and end of the span and underlines everything in between, so this works.
#[derive(Copy, Clone)]
pub struct FullSpan(Span, Span);

impl FullSpan {
    pub fn from_span(span: Span) -> Self {
        Self(span, span)
    }
    pub fn from_spanned<T: ToTokens + syn::spanned::Spanned>(span: &T) -> Self {
        let start = span.span();
        let end = span
            .to_token_stream()
            .into_iter()
            .last()
            .map(|t| t.span())
            .unwrap_or(start);
        Self(start, end)
    }
    pub fn apply(self, a: TokenStream, b: TokenStream) -> TokenStream {
        let mut ret = a.with_span(self.0);
        ret.extend(b.with_span(self.1));
        ret
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

/// Format a list of items as a comma-separated list, with "or" before the last item.
pub fn list_items<T>(items: &[T], mut display: impl FnMut(&T) -> String) -> String {
    match items {
        [] => String::new(),
        [x] => display(x),
        [a, b] => format!("{} or {}", display(a), display(b)),
        [start @ .., last] => {
            let mut s = String::new();
            for item in start {
                s += &display(item);
                s += ", ";
            }
            s += "or ";
            s += &display(last);
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
