use std::fmt::{Display, Write};

use proc_macro2::{Literal, Span};
use unicode_width::UnicodeWidthStr;

use crate::*;

/// A wrapper around a string literal
pub struct StrLit {
    text: String,
    span_provider: Literal,
}

impl StrLit {
    pub fn new(input: syn::LitStr) -> Self {
        // the full string with any ", r", r#", ... prefix and suffix
        let text = input.to_token_stream().to_string();

        // input has to be parsed as `syn::LitStr` to access the content as a string. But in order to
        // call subspan, we need it as a `proc_macro2::Literal`. So: parse it as `LitStr` first and
        // convert that to a `Literal` with the same content and span.
        let mut span_provider = Literal::string(&text);
        span_provider.set_span(input.span()); // input is a single Token so span() works even on stable

        Self {
            text,
            span_provider,
        }
    }

    pub fn is_raw(&self) -> bool {
        self.text.starts_with('r')
    }

    pub fn to_slice<'a>(&'a self) -> StrLitSlice<'a> {
        // find the position of the opening quote. raw strings may have a prefix of any length,
        // which needs to be skipped. This information used to be provided by syn, but was removed
        // at some point. This approach is a dirty hack, which relies on the
        // .to_token_stream().to_string() call in the Parse impl below returning the full string
        // with the prefix intact, so that we can parse it ourselves.
        // it also requires all strings to start with a " character, which they should?
        let quote_position = self.text.find('"').unwrap();
        let prefix_length = quote_position + '"'.len_utf8();

        let suffix_length = if prefix_length > 1 {
            // raw strings have a suffix of the same length as the prefix, but without the 'r'
            prefix_length - 'r'.len_utf8()
        } else {
            // non-raw strings only have the " suffix
            '"'.len_utf8()
        };

        let start = prefix_length;
        let end = self.text.len() - suffix_length;

        StrLitSlice {
            src: self,
            start,
            end,
        }
    }
}

/// A slice into [`StrLit`]
#[derive(Clone, Copy)]
pub struct StrLitSlice<'a> {
    src: &'a StrLit,
    start: usize,
    end: usize,
}

impl<'a> StrLitSlice<'a> {
    pub fn text(&self) -> &str {
        &self.src.text[self.start..self.end]
    }
    #[expect(unused, reason = "TODO: used once other todos are done")]
    pub fn is_raw(&self) -> bool {
        self.src.is_raw()
    }

    #[track_caller]
    pub fn slice<R>(&'_ self, range: R) -> StrLitSlice<'a>
    where
        R: std::ops::RangeBounds<usize>
            + std::slice::SliceIndex<str, Output = str>
            + Clone
            + std::fmt::Debug,
    {
        use std::ops::Bound::*;
        let start = match range.start_bound() {
            Included(&start) => start,
            Excluded(_) => unimplemented!("StrLitSlice::slice: excluded start"),
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Included(&end) => end + self.text()[end..].chars().next().unwrap().len_utf8(),
            Excluded(&end) => end,
            Unbounded => self.end - self.start,
        };

        let text = self.text().get(start..end).unwrap_or_else(|| {
            panic!(
                "StrLitSlice::slice: invalid range {:?} for {:?}",
                range,
                self.text()
            );
        });
        assert!(!text.is_empty(), "StrLitSlice::slice: empty slice");

        let mut ret = *self;
        ret.start += start;
        ret.end = ret.start + (end - start);
        ret
    }

    /// Provides a span for the slice if possible. Otherwise, returns the entire span.
    pub fn span(&self) -> Span {
        self.src
            .span_provider
            .subspan(self.start..self.end)
            .unwrap_or_else(|| self.src.span_provider.span())
    }

    /// Generates a `Result::Err` with the given message for the slice.
    pub fn err<T, E: From<Error>>(&self, message: impl Display) -> std::result::Result<T, E> {
        Err(self.error(message).into())
    }

    /// Generates a [`crate::Error`] with the given message for the slice.
    pub fn error(&self, message: impl Display) -> Error {
        // subspan allows pointing at a span that is not the whole string, but it only works in nightly
        if let Some(span) = self.src.span_provider.subspan(self.start..self.end) {
            Error::new(span, message)
        } else {
            // Workaround for stable: print a copy of the entire format string into the error message
            // and manually underline the desired section.
            let mut m = String::new();
            writeln!(m, "{message}:").unwrap(); // TODO: split by lines

            let text_prefix = "At ";
            let text_prefix_len = 3; // length of "At "

            writeln!(m, "{}{}", text_prefix, self.src.text).unwrap();

            let squiggle_start = UnicodeWidthStr::width(&self.src.text[..self.start]);
            let squiggle_len = UnicodeWidthStr::width(self.text());

            // Add the line with the error squiggles
            // start already includes the string prefix
            for _ in 0..(text_prefix_len + squiggle_start) {
                m.push(' ');
            }
            for _ in 0..squiggle_len {
                m.push('^');
            }
            Error::new_spanned(&self.src.span_provider, m)
        }
    }
}

impl Parse for StrLit {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse().map(Self::new)
    }
}

impl ToTokens for StrLit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.span_provider.to_tokens(tokens);
    }
}

impl std::ops::Deref for StrLit {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl Display for StrLit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}
impl Display for StrLitSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text())
    }
}
