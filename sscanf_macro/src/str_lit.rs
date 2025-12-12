use std::fmt::{Display, Write};

use proc_macro2::{Literal, Span};

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

        const NO_QUOTE_MSG: &str = r"sscanf: Encountered a string literal without quotes, which is not supported.
If Rust added something like this since the last update of sscanf, please open an issue here: <https://github.com/mich101mich/sscanf/issues>";
        const INVALID_RAW_MSG: &str = r"sscanf: Encountered an invalid raw string literal, which is not supported.
If Rust added something like this since the last update of sscanf, please open an issue here: <https://github.com/mich101mich/sscanf/issues>";

        const R_LEN: usize = 'r'.len_utf8(); // == 1, but written like this for clarity
        const QUOTE_LEN: usize = '"'.len_utf8();

        let quote_position = self.text.find('"').expect(NO_QUOTE_MSG);
        let prefix_length = quote_position + QUOTE_LEN;

        let suffix_length = if quote_position == 0 {
            // non-raw strings only have the " suffix

            assert!(
                self.text.ends_with('"'),
                "{NO_QUOTE_MSG}\nOffending string: {}",
                self.text
            );

            QUOTE_LEN
        } else {
            // raw strings have a suffix of the same length as the prefix, but without the 'r'
            let num_hashtags = quote_position - R_LEN;

            assert!(self.text.starts_with('r'), "{INVALID_RAW_MSG}");

            let prefix_hashtags = &self.text[R_LEN..][..num_hashtags];
            assert!(
                prefix_hashtags.chars().all(|c| c == '#'),
                r#"{INVALID_RAW_MSG}
Found invalid characters in raw string prefix of offending string: {prefix_hashtags}
Offending string: {}"#,
                self.text
            );

            let suffix_hashtags = &self.text[self.text.len() - num_hashtags..];
            assert!(
                suffix_hashtags.chars().all(|c| c == '#'),
                r#"{INVALID_RAW_MSG}
Found invalid characters in raw string suffix of offending string: {suffix_hashtags}
Offending string: {}"#,
                self.text
            );

            QUOTE_LEN + num_hashtags
        };

        assert!(
            self.text.len() >= prefix_length + suffix_length,
            r#"sscanf: Unsupported string literal.
Offending string: {}"#,
            self.text
        );

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
            Included(&end) => {
                let Some(next_char) = self.text()[end..].chars().next() else {
                    panic!(
                        "StrLitSlice::slice: invalid range {range:?} for {:?}",
                        self.text()
                    );
                };
                end + next_char.len_utf8()
            }
            Excluded(&end) => end,
            Unbounded => self.end - self.start,
        };

        assert!(
            self.text().get(start..end).is_some(),
            "StrLitSlice::slice: invalid range {range:?} for {:?}",
            self.text()
        );

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
            return Error::new(span, message);
        }

        // Workaround for stable: print a copy of the entire format string into the error message
        // and manually underline the desired section.

        // Rust's (current) output looks like this:
        //  --> tests/fail/nightly/missing_type.rs:3:43
        //   |
        // 3 |     sscanf::sscanf!("hi", "asdf{}{usize}as{}df{i32}", usize);
        //   |                                           ^^

        let src_span = self.src.span_provider.span();
        let file = src_span.stable_file();

        let prefix = &self.src.text[..self.start];
        let self_text = &self.src.text[self.start..self.end];
        let suffix = &self.src.text[self.end..];

        let self_lines = self_text.split_inclusive('\n').collect::<Vec<_>>(); // split_inclusive to preserve \r and \n at the end of each line

        let start_line = src_span.stable_line() + prefix.lines().count() - 1;
        let end_line = start_line + self_lines.len() - 1;

        let ln_length = end_line.to_string().len(); // line number length for formatting
        let ln_blank = " ".repeat(ln_length); // blank space for line number column

        let (prefix, column_offset) =
            if let Some((_previous_lines, prefix)) = prefix.rsplit_once('\n') {
                (prefix, 1) // 1: we are on our own line, but with 1-based column indexing
            } else {
                (prefix, src_span.stable_column())
            };
        let (prefix, prefix_len) = rust_compiler_replacements(prefix);
        let column = column_offset + prefix_len;

        let suffix = suffix.lines().next().unwrap_or(""); // whatever part of the suffix is on the same line
        let suffix = rust_compiler_replacements(suffix).0;

        const E: &str = ""; // empty string so that we can use the formatting width specifier to create repeated characters

        let mut m = String::new();
        writeln!(m, "{message}:").unwrap();
        writeln!(m, "{ln_blank}--> {file}:{start_line}:{column}").unwrap();
        writeln!(m, "{ln_blank} |").unwrap();

        if self_lines.len() <= 1 {
            let line = self_lines.first().copied().unwrap_or("");
            let (line, line_len) = rust_compiler_replacements(line);
            writeln!(m, "{start_line: >ln_length$} | {prefix}{line}{suffix}").unwrap();
            writeln!(m, "{ln_blank} | {E: <prefix_len$}{E:^<line_len$}").unwrap(); // spaces for prefix, then '^' for the string part
        } else {
            //   --> tests/fail/nightly/multiline_format_str.rs:12:10
            //    |
            // 12 |           "{This
            //    |  __________^
            // 13 | | is another faulty
            // 14 | | multiline string!"
            //    | |_________________^

            // we have at least two lines, so we can safely split first and last. middle_lines may be empty.
            let (first_line, rest_lines) = self_lines.split_first().unwrap();
            let (last_line, middle_lines) = rest_lines.split_last().unwrap();

            write!(m, "{start_line: >ln_length$} |   {prefix}{first_line}").unwrap(); // no newline, since first_line contains any \r and \n from the original
            writeln!(m, "{ln_blank} |  _{E:_<prefix_len$}^").unwrap();

            if middle_lines.len() <= 4 {
                for (i, line) in middle_lines.iter().enumerate() {
                    let line_no = start_line + 1 + i;
                    write!(m, "{line_no: >ln_length$} | | {line}").unwrap();
                }
            } else {
                for (i, line) in middle_lines.iter().enumerate().take(2) {
                    let line_no = start_line + 1 + i;
                    write!(m, "{line_no: >ln_length$} | | {line}").unwrap();
                }

                writeln!(m, "{ln_blank} | | ...").unwrap();
                // Note: The rust compiler would write "... |" instead, but if we do that in the error message itself, it somehow
                // breaks the indentation of the previous line.

                let line = middle_lines.last().unwrap();
                let line_no = end_line - 1;
                write!(m, "{line_no: >ln_length$} | | {line}").unwrap();
            }

            let (last_line, last_line_len) = rust_compiler_replacements(last_line);
            writeln!(m, "{end_line: >ln_length$} | | {last_line}{suffix}").unwrap();
            writeln!(m, "{ln_blank} | |{E:_<last_line_len$}^").unwrap();
        }

        Error::new_spanned(&self.src.span_provider, m)
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

fn rust_compiler_replacements(input: &str) -> (String, usize) {
    // When the Rust compiler prints strings in error messages, it replaces certain characters
    // (mostly control characters) with visible representations for consistency. Since our error
    // will be printed as part of a rust error message, it needs to match this behavior to ensure that
    // the underlining aligns correctly.
    // See <https://github.com/rust-lang/rust/blob/main/compiler/rustc_errors/src/emitter.rs>
    // and <https://github.com/rust-lang/rust/blob/main/compiler/rustc_span/src/lib.rs> function `char_width`

    let mut output = String::new();
    let mut length = 0;
    for c in input.chars() {
        let (replacement, width) = match c {
            '\0' => ("␀", 1),
            '\u{0001}' => ("␁", 1),
            '\u{0002}' => ("␂", 1),
            '\u{0003}' => ("␃", 1),
            '\u{0004}' => ("␄", 1),
            '\u{0005}' => ("␅", 1),
            '\u{0006}' => ("␆", 1),
            '\u{0007}' => ("␇", 1),
            '\u{0008}' => ("␈", 1),
            '\t' => ("    ", 4),
            '\u{000b}' => ("␋", 1),
            '\u{000c}' => ("␌", 1),
            '\u{000d}' => ("␍", 1),
            '\u{000e}' => ("␎", 1),
            '\u{000f}' => ("␏", 1),
            '\u{0010}' => ("␐", 1),
            '\u{0011}' => ("␑", 1),
            '\u{0012}' => ("␒", 1),
            '\u{0013}' => ("␓", 1),
            '\u{0014}' => ("␔", 1),
            '\u{0015}' => ("␕", 1),
            '\u{0016}' => ("␖", 1),
            '\u{0017}' => ("␗", 1),
            '\u{0018}' => ("␘", 1),
            '\u{0019}' => ("␙", 1),
            '\u{001a}' => ("␚", 1),
            '\u{001b}' => ("␛", 1),
            '\u{001c}' => ("␜", 1),
            '\u{001d}' => ("␝", 1),
            '\u{001e}' => ("␞", 1),
            '\u{001f}' => ("␟", 1),
            '\u{007f}' => ("␡", 1),
            '\u{200d}' => ("", 1),
            '\u{202a}' => ("�", 1),
            '\u{202b}' => ("�", 1),
            '\u{202c}' => ("�", 1),
            '\u{202d}' => ("�", 1),
            '\u{202e}' => ("�", 1),
            '\u{2066}' => ("�", 1),
            '\u{2067}' => ("�", 1),
            '\u{2068}' => ("�", 1),
            '\u{2069}' => ("�", 1),
            _ => {
                output.push(c);
                length += unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
                continue;
            }
        };
        output.push_str(replacement);
        length += width;
    }

    (output, length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_lit_slice_basic() {
        let tokens = quote! { "Hello, world!" };
        let str_lit: StrLit = syn::parse2(tokens).unwrap();

        assert_eq!(str_lit.text, "\"Hello, world!\""); // syn adds the quotes back in.
        assert!(!str_lit.is_raw());

        let slice = str_lit.to_slice();
        assert_eq!(slice.text(), "Hello, world!");
        assert_eq!(slice.start, 1); // skip opening "
        assert_eq!(slice.end, str_lit.text.len() - 1); // skip closing "

        let sub_slice = slice.slice(7..12);
        assert_eq!(sub_slice.text(), "world");
        assert_eq!(sub_slice.start, 8); // +1 for opening "
        assert_eq!(sub_slice.end, 13);
    }

    #[test]
    fn str_lit_slice_raw() {
        let tokens = quote! { r"Raw string" };
        let str_lit: StrLit = syn::parse2(tokens).unwrap();

        assert_eq!(str_lit.text, "r\"Raw string\"");
        assert!(str_lit.is_raw());

        let slice = str_lit.to_slice();
        assert_eq!(slice.text(), "Raw string");
        assert_eq!(slice.start, 2); // skip r"
        assert_eq!(slice.end, str_lit.text.len() - 1); // skip closing "
    }

    #[test]
    fn str_lit_slice_raw_extended() {
        let tokens = quote! { r#"Raw string with "quotes" and \backslashes\"# };
        let str_lit: StrLit = syn::parse2(tokens).unwrap();

        assert_eq!(
            str_lit.text,
            "r#\"Raw string with \"quotes\" and \\backslashes\\\"#"
        );
        assert!(str_lit.is_raw());

        let slice = str_lit.to_slice();
        assert_eq!(
            slice.text(),
            "Raw string with \"quotes\" and \\backslashes\\"
        );
        assert_eq!(slice.start, 3); // skip r#"
        assert_eq!(slice.end, str_lit.text.len() - 2); // skip closing "#
    }

    #[test]
    fn advanced_unicode_support() {
        let tokens = quote! { "y̆😛y̆{Ay̆y̆y̆:😛}y̆😛y̆" };
        let str_lit: StrLit = syn::parse2(tokens).unwrap();

        let (converted, full_length) = rust_compiler_replacements(&str_lit.text);
        assert_eq!(
            converted,
            "\"y\u{306}😛y\u{306}{Ay\u{306}y\u{306}y\u{306}:😛}y\u{306}😛y\u{306}\""
        );
        assert_eq!(full_length, 19);

        let tokens = quote! { r##"y̆👨‍👩‍👧‍👦y̆{Ay̆y̆y̆:😛}y̆😛y̆"## };
        let str_lit: StrLit = syn::parse2(tokens).unwrap();

        let (converted, full_length) = rust_compiler_replacements(&str_lit.text);
        assert_eq!(
            converted,
            "r##\"y\u{306}👨👩👧👦y\u{306}{Ay\u{306}y\u{306}y\u{306}:😛}y\u{306}😛y\u{306}\"##"
        );
        assert_eq!(full_length, 33);
    }
}
