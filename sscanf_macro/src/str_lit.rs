use std::fmt::Write;

use proc_macro2::{Literal, Span};
use quote::ToTokens;
use syn::{parse::ParseBuffer, LitStr};

use crate::{Error, Result};

/// A wrapper around a string literal
pub struct StrLit {
    pub text: String,
    span_provider: Literal,
    span_offset: usize,
}

/// A slice into StrLit
pub struct StrLitSlice<'a> {
    src: &'a StrLit,
    pub text: &'a str,
    range: std::ops::Range<usize>,
}

impl StrLit {
    pub fn to_slice(&self) -> StrLitSlice {
        StrLitSlice {
            src: self,
            text: &self.text,
            range: self.span_offset..self.text.chars().count() + self.span_offset,
        }
    }
}

impl<'a> StrLitSlice<'a> {
    #[track_caller]
    pub fn slice<R>(&'_ self, range: R) -> StrLitSlice<'a>
    where
        R: std::ops::RangeBounds<usize> + std::slice::SliceIndex<str, Output = str>,
    {
        let char_len = self.range.len();
        use std::ops::Bound::*;
        let start = match range.start_bound() {
            Included(&n) => n,
            Excluded(&n) => n + 1,
            Unbounded => 0,
        };
        if start >= char_len {
            panic!(
                "range start index {} out of range for StrLitSlice of length {}",
                start, char_len
            );
        }
        let end = match range.end_bound() {
            Included(&n) => n + 1,
            Excluded(&n) => n,
            Unbounded => char_len,
        };
        if end > char_len {
            panic!(
                "range end index {} out of range for StrLitSlice of length {}",
                end, char_len
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
        let mut iter = self.text.char_indices();
        let byte_start = iter
            .nth(start)
            .unwrap_or_else(|| panic!("a:{}:{}", file!(), line!()))
            .0;
        let byte_end = if end < char_len {
            iter.nth(end - start - 1)
                .unwrap_or_else(|| panic!("a:{}:{}", file!(), line!()))
                .0
        } else {
            self.text.len()
        };
        let span_start = start + self.range.start;
        let span_end = end + self.range.start;
        StrLitSlice {
            src: self.src,
            text: &self.text[byte_start..byte_end],
            range: span_start..span_end,
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

    /// Generates a syn::Error with the given message for the slice.
    pub fn error(&self, message: &str) -> Error {
        // subspan allows pointing at a span that is not the whole string, but it only works in nightly
        if let Some(span) = self.src.span_provider.subspan(self.range.clone()) {
            Error::new(span, message)
        } else {
            // Workaround for stable: print a copy of the entire format string into the error message
            // and manually underline the desired section.
            let mut m = String::new();
            writeln!(m, "{}:", message).unwrap_or_else(|_| panic!("a:{}", line!()));

            // Add the line with the format string
            if self.src.span_offset > 1 {
                let hashtags = std::iter::repeat('#')
                    .take(self.src.span_offset - 2) // - the 'r' and '"'
                    .collect::<String>();

                writeln!(m, "At r{0}\"{1}\"{0}", hashtags, self.src.text)
                    .unwrap_or_else(|_| panic!("a:{}", line!()));
            } else {
                writeln!(m, "At \"{}\"", self.src.text).unwrap_or_else(|_| panic!("a:{}", line!()));
            }

            // Add the line with the error squiggles
            // start already includes the span_offset, so only the "At " prefix is missing
            for _ in 0..("At ".len() + self.range.start) {
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

        let span_offset = {
            // this is a dirty hack to see if the literal is a raw string, which is necessary for
            // subspan to skip the 'r', 'r#', ...
            // this should be a thing that I can check on LitStr or Literal or whatever, but nooo,
            // I have to print it into a String via a TokenStream and check that one -.-
            //
            // "fun" fact: This used to be easier in syn 0.11 where `Lit::Str` gave _two_ values:
            // the literal and a `StrStyle` which looked like this:
            // enum StrStyle {
            //     Cooked,      // a non-raw string
            //     Raw(usize),
            // }       ^^^^^ See this usize here? That's the number of '#' in the prefix
            //               which is _exactly_ what I'm trying to calculate here! How convenient!
            // This was apparently removed at some point for being TOO USEFUL.
            let lit = fmt.to_token_stream().to_string();
            // Yes, this is the easiest way to solve this. Have syn read the Rust Code (which used
            // to be a string) as a TokenStream to parse the LitStr, turn that back into a
            // TokenStream and then into a String (LitStr cannot be directly converted to a String)
            // and then iterate over that string for the first ", because anything before that
            // **should** (this will totally break at some point) be the prefix.
            lit.chars()
                .position(|c| c == '"')
                .unwrap_or_else(|| panic!("a:{}:{}", file!(), line!()))
                + 1 // + 1 for the " itself
        };

        // fmt has to be parsed as `syn::LitStr` to access the content as a string. But in order to
        // call subspan, we need it as a `proc_macro2::Literal`. So: parse it as `LitStr` first and
        // convert that to a `Literal` with the same content and span.
        let mut span_provider = Literal::string(&fmt.value());
        span_provider.set_span(fmt.span()); // fmt is a single Token so span() works even on stable
        Ok(Self {
            text: fmt.value(),
            span_provider,
            span_offset,
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
