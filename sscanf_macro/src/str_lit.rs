use std::fmt::Write;

use proc_macro2::{Literal, Span};
use quote::ToTokens;
use syn::{parse::ParseBuffer, LitStr};
use unicode_segmentation::UnicodeSegmentation;

use crate::{Error, Result};

/// A wrapper around a string literal
pub struct StrLit {
    text: String,
    /// The indices of the grapheme clusters in `text`. Contains a past-the-end index at the end.
    grapheme_indices: Vec<usize>,
    span_provider: Literal,
}

impl StrLit {
    pub fn graphemes(&self) -> impl Iterator<Item = &str> {
        self.grapheme_indices
            .windows(2)
            .map(move |w| &self.text[w[0]..w[1]])
    }
    pub fn to_slice(&self) -> StrLitSlice {
        // find the position of the opening quote. raw strings may have a prefix of any length,
        // which needs to be skipped. This information used to be provided by syn, but was removed
        // at some point. This approach is a dirty hack, which relies on the
        // .to_token_stream().to_string() call in the Parse impl below returning the full string
        // with the prefix intact, so that we can parse it ourselves.
        // it also requires all strings to start with a " character, which they should?
        let prefix_length = self.graphemes().position(|s| s == "\"").unwrap() + 1; // +1 to include the quote itself

        let suffix_length = if prefix_length > 1 {
            // raw strings have a suffix of the same length as the prefix, but without the 'r'
            prefix_length - 1
        } else {
            // non-raw strings only have the " suffix
            1
        };

        let range_start = prefix_length;
        let range_end = self.grapheme_indices.len() - 1 - suffix_length; // -1 for the past-the-end index

        let byte_start = self.grapheme_indices[range_start];
        let byte_end = self.grapheme_indices[range_end];

        StrLitSlice {
            src: self,
            text: &self.text[byte_start..byte_end],
            range: range_start..range_end,
        }
    }
}

/// A slice into StrLit
#[derive(Clone)]
pub struct StrLitSlice<'a> {
    src: &'a StrLit,
    pub text: &'a str,
    range: std::ops::Range<usize>,
}

impl<'a> StrLitSlice<'a> {
    #[track_caller]
    pub fn slice<R>(&'_ self, range: R) -> StrLitSlice<'a>
    where
        R: std::ops::RangeBounds<usize> + std::slice::SliceIndex<str, Output = str>,
    {
        let num_graphemes = self.range.len();
        use std::ops::Bound::*;
        let start = match range.start_bound() {
            Included(&n) => n,
            Excluded(&n) => n + 1,
            Unbounded => 0,
        };
        if start >= num_graphemes {
            panic!(
                "range start index {} out of range for StrLitSlice of length {}",
                start, num_graphemes
            );
        }
        let end = match range.end_bound() {
            Included(&n) => n + 1,
            Excluded(&n) => n,
            Unbounded => num_graphemes,
        };
        if end > num_graphemes {
            panic!(
                "range end index {} out of range for StrLitSlice of length {}",
                end, num_graphemes
            );
        }
        if start > end {
            panic!(
                "range start index {} is greater than end index {}",
                start, end
            );
        } else if start == end {
            panic!("StrLitSlice must not be empty");
        }

        let range_start = self.range.start + start;
        let range_end = self.range.start + end;

        let byte_start = self.src.grapheme_indices[range_start];
        let byte_end = self.src.grapheme_indices[range_end]; // works because of the past-the-end index

        StrLitSlice {
            src: self.src,
            text: &self.src.text[byte_start..byte_end],
            range: range_start..range_end,
        }
    }

    /// Provides a span for the slice if possible. Otherwise, returns the entire span.
    pub fn span(&self) -> Span {
        self.src
            .span_provider
            .subspan(self.range.clone())
            .unwrap_or_else(|| self.src.span_provider.span())
    }

    /// Generates a Result::Err with the given message for the slice.
    pub fn err<T>(&self, message: &str) -> Result<T> {
        Err(self.error(message))
    }

    /// Generates a crate::Error with the given message for the slice.
    pub fn error(&self, message: &str) -> Error {
        // subspan allows pointing at a span that is not the whole string, but it only works in nightly
        if let Some(span) = self.src.span_provider.subspan(self.range.clone()) {
            Error::new(span, message)
        } else {
            // Workaround for stable: print a copy of the entire format string into the error message
            // and manually underline the desired section.
            let mut m = String::new();
            writeln!(m, "{}:", message).unwrap();

            let text_prefix = "At ";
            let text_prefix_len = text_prefix.graphemes(true).count();

            writeln!(m, "{}{}", text_prefix, self.src.text).unwrap();

            // Add the line with the error squiggles
            // start already includes the string prefix
            for _ in 0..(text_prefix_len + self.range.start) {
                m.push(' ');
            }
            for _ in self.range.clone() {
                m.push('^');
            }
            Error::new_spanned(&self.src.span_provider, m)
        }
    }
}

impl syn::parse::Parse for StrLit {
    fn parse(input: &ParseBuffer<'_>) -> syn::Result<Self> {
        let fmt: LitStr = input.parse()?;

        // the full string with any ", r", r#", ... prefix and suffix
        let text = fmt.to_token_stream().to_string();

        // fmt has to be parsed as `syn::LitStr` to access the content as a string. But in order to
        // call subspan, we need it as a `proc_macro2::Literal`. So: parse it as `LitStr` first and
        // convert that to a `Literal` with the same content and span.
        let mut span_provider = Literal::string(&text);
        span_provider.set_span(fmt.span()); // fmt is a single Token so span() works even on stable

        let mut grapheme_indices = text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        // Add the end of the string as a grapheme index to make the slice function easier
        grapheme_indices.push(text.len());

        Ok(Self {
            text,
            grapheme_indices,
            span_provider,
        })
    }
}

use std::ops::*;

impl Deref for StrLit {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}
