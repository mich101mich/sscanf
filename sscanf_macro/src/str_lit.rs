use std::fmt::{Display, Write};

use proc_macro2::{Literal, Span};

use crate::*;

/// A wrapper around a string literal
pub struct StrLit {
    text: String,
    span_provider: Literal,
}

impl StrLit {
    pub fn new(input: &syn::LitStr) -> Self {
        Self {
            text: input.to_token_stream().to_string(), // the full string with any ", r", r#", ... prefix and suffix
            span_provider: input.token(),
        }
    }
    pub fn from_parts(inner_text: &str, span: Span) -> Self {
        let text = format!("\"{inner_text}\""); // add quotes around the inner text
        let mut span_provider = Literal::string(&text);
        span_provider.set_span(span);
        Self {
            text,
            span_provider,
        }
    }

    pub fn is_raw(&self) -> bool {
        self.text.starts_with('r')
    }

    pub fn to_slice(&self) -> StrLitSlice<'_> {
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
                r"{INVALID_RAW_MSG}
Found invalid characters in raw string prefix of offending string: {prefix_hashtags}
Offending string: {}",
                self.text
            );

            let suffix_hashtags = &self.text[self.text.len() - num_hashtags..];
            assert!(
                suffix_hashtags.chars().all(|c| c == '#'),
                r"{INVALID_RAW_MSG}
Found invalid characters in raw string suffix of offending string: {suffix_hashtags}
Offending string: {}",
                self.text
            );

            QUOTE_LEN + num_hashtags
        };

        assert!(
            self.text.len() >= prefix_length + suffix_length,
            r"sscanf: Unsupported string literal.
Offending string: {}",
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

        if start > self.text().len() {
            panic!(
                "StrLitSlice::slice: invalid range {range:?} for {:?}: start index out of bounds of {} bytes",
                self.text(),
                self.text().len()
            );
        }
        if self.text().get(start..).is_none() {
            panic!(
                "StrLitSlice::slice: invalid range {range:?} for {:?}: start index not at char boundary",
                self.text()
            );
        }

        let end = match range.end_bound() {
            Included(&end) => {
                if end >= self.text().len() {
                    panic!(
                        "StrLitSlice::slice: invalid range {range:?} for {:?}: end index out of bounds of {} bytes",
                        self.text(),
                        self.text().len()
                    );
                }
                let Some(after) = self.text().get(end..) else {
                    panic!(
                        "StrLitSlice::slice: invalid range {range:?} for {:?}: end index not at char boundary",
                        self.text()
                    );
                };
                let next_char = after.chars().next().unwrap(); // safe: we already checked end < len
                end + next_char.len_utf8()
            }
            Excluded(&end) => end,
            Unbounded => self.end - self.start,
        };

        if start > end {
            panic!(
                "StrLitSlice::slice: invalid range {range:?} for {:?}: start > end",
                self.text()
            );
        }

        if end > self.text().len() {
            panic!(
                "StrLitSlice::slice: invalid range {range:?} for {:?}: end index out of bounds of {} bytes",
                self.text(),
                self.text().len()
            );
        }
        if self.text().get(..end).is_none() {
            panic!(
                "StrLitSlice::slice: invalid range {range:?} for {:?}: end index not at char boundary",
                self.text()
            );
        }

        assert!(
            self.text().get(start..end).is_some(),
            "StrLitSlice::slice: invalid range {range:?} for {:?}",
            self.text()
        ); // should be unreachable due to the checks above

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
            if line.ends_with('\n') {
                write!(m, "{start_line: >ln_length$} | {prefix}{line}").unwrap();
            } else {
                writeln!(m, "{start_line: >ln_length$} | {prefix}{line}{suffix}").unwrap();
            }
            // spaces for prefix, then '^' for the string part
            writeln!(m, "{ln_blank} | {E: <prefix_len$}{E:^<line_len$}").unwrap();
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

                // write "...   |" padded to align the `|` with the right `|` of the numbered lines
                let total_len = ln_length + 3; // line number + " | "
                writeln!(m, "{: <total_len$}|", "...").unwrap();

                let line = middle_lines.last().unwrap();
                let line_no = end_line - 1;
                write!(m, "{line_no: >ln_length$} | | {line}").unwrap();
            }

            let (last_line, last_line_len) = rust_compiler_replacements(last_line);
            if last_line.ends_with('\n') {
                write!(m, "{end_line: >ln_length$} | | {last_line}").unwrap();
            } else {
                writeln!(m, "{end_line: >ln_length$} | | {last_line}{suffix}").unwrap();
            }
            writeln!(m, "{ln_blank} | |{E:_<last_line_len$}^").unwrap();
        }

        Error::new_spanned(&self.src.span_provider, m)
    }
}

impl Parse for StrLit {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse().map(|input| Self::new(&input))
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
impl std::fmt::Debug for StrLit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StrLit({})", self.text)
    }
}
impl Display for StrLitSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text())
    }
}
impl std::fmt::Debug for StrLitSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StrLitSlice({})", self.text())
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
            '\u{202a}' | '\u{202b}' | '\u{202c}' | '\u{202d}' | '\u{202e}' | '\u{2066}'
            | '\u{2067}' | '\u{2068}' | '\u{2069}' => ("�", 1),
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

    #[macro_export]
    macro_rules! str_lit {
        ( $input:literal ) => {
            syn::parse2::<StrLit>(quote::quote! { $input }).unwrap()
        };
        ( # $input:ident ) => {
            syn::parse2::<StrLit>(quote::quote! { # $input }).unwrap()
        };
    }

    #[test]
    fn str_lit_slice_basic() {
        let str_lit = str_lit! { "Hello, world! 😄" };

        assert_eq!(str_lit.text, "\"Hello, world! 😄\""); // syn adds the quotes back in.
        assert!(!str_lit.is_raw());

        let slice = str_lit.to_slice();
        assert_eq!(slice.text(), "Hello, world! 😄");
        assert_eq!(slice.start, 1); // skip opening "
        assert_eq!(slice.end, str_lit.text.len() - 1); // skip closing "

        let sub_slice = slice.slice(7..12);
        assert_eq!(sub_slice.text(), "world");
        assert_eq!(sub_slice.start, 8); // +1 for opening "
        assert_eq!(sub_slice.end, 13);

        let sub_slice_inclusive = slice.slice(7..=11);
        assert_eq!(sub_slice_inclusive.text(), "world");
        assert_eq!(sub_slice_inclusive.start, 8); // +1 for opening "
        assert_eq!(sub_slice_inclusive.end, 13);

        let sub_slice_unbounded_start = slice.slice(..5);
        assert_eq!(sub_slice_unbounded_start.text(), "Hello");
        assert_eq!(sub_slice_unbounded_start.start, 1); // +1 for opening "
        assert_eq!(sub_slice_unbounded_start.end, 6);

        let sub_slice_unbounded_end = slice.slice(7..);
        assert_eq!(sub_slice_unbounded_end.text(), "world! 😄");
        assert_eq!(sub_slice_unbounded_end.start, 8); // +1 for opening "
        assert_eq!(sub_slice_unbounded_end.end, str_lit.text.len() - 1);

        let sub_slice_fully_unbounded = slice.slice(..);
        assert_eq!(sub_slice_fully_unbounded.text(), slice.text());
        assert_eq!(sub_slice_fully_unbounded.start, slice.start);
        assert_eq!(sub_slice_fully_unbounded.end, slice.end);

        let sub_slice_multibyte = slice.slice(7..18); // emoji is at 14..18
        assert_eq!(sub_slice_multibyte.text(), "world! 😄");

        let sub_slice_inclusive_multibyte = slice.slice(7..=14);
        assert_eq!(sub_slice_inclusive_multibyte.text(), "world! 😄");
    }

    #[test]
    fn str_lit_slice_raw() {
        let str_lit = str_lit! { r"Raw string" };

        assert_eq!(str_lit.text, "r\"Raw string\"");
        assert!(str_lit.is_raw());

        let slice = str_lit.to_slice();
        assert_eq!(slice.text(), "Raw string");
        assert_eq!(slice.start, 2); // skip r"
        assert_eq!(slice.end, str_lit.text.len() - 1); // skip closing "
    }

    #[test]
    fn str_lit_slice_raw_extended() {
        let str_lit = str_lit! { r#"Raw string with "quotes" and \backslashes\"# };

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
        let str_lit = str_lit! { "y̆😛y̆{Ay̆y̆y̆:😛}y̆😛y̆" };

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

    #[test]
    fn str_lit_slice_invalid_range() {
        let str_lit = str_lit! { "Hello, world! 😄" };
        let slice = str_lit.to_slice();

        assert_panic_message_eq!(
            slice.slice(7..20),
            "StrLitSlice::slice: invalid range 7..20 for \"Hello, world! 😄\": end index out of bounds of 18 bytes"
        );
        assert_panic_message_eq!(
            slice.slice(7..=20),
            "StrLitSlice::slice: invalid range 7..=20 for \"Hello, world! 😄\": end index out of bounds of 18 bytes"
        );

        assert_panic_message_eq!(
            slice.slice(20..25),
            "StrLitSlice::slice: invalid range 20..25 for \"Hello, world! 😄\": start index out of bounds of 18 bytes"
        );

        assert_panic_message_eq!(
            #[allow(clippy::reversed_empty_ranges)] // yes clippy, I know. Good catch though!
            slice.slice(12..10),
            "StrLitSlice::slice: invalid range 12..10 for \"Hello, world! 😄\": start > end"
        );

        assert_panic_message_eq!(
            slice.slice(7..15),
            "StrLitSlice::slice: invalid range 7..15 for \"Hello, world! 😄\": end index not at char boundary"
        );

        assert_panic_message_eq!(
            slice.slice(8..=15),
            "StrLitSlice::slice: invalid range 8..=15 for \"Hello, world! 😄\": end index not at char boundary"
        );

        assert_panic_message_eq!(
            slice.slice(15..),
            "StrLitSlice::slice: invalid range 15.. for \"Hello, world! 😄\": start index not at char boundary"
        );
    }

    #[test]
    fn test_error_fallback() {
        let str_lit = str_lit! { "Hello, world! 😄" };
        let sub_slice = str_lit.to_slice().slice(7..12);

        let err = sub_slice.error("Test error message").to_string();
        assert_eq!(
            err,
            r#"Test error message:
   --> /inside/a/test.rs:999:131
    |
999 | "Hello, world! 😄"
    |         ^^^^^
"#
        );
    }

    #[test]
    fn test_error_fallback_multiline() {
        let multiline_str_lit = str_lit! { r#"Line 1
Line 2
Line 3
Line 4
Line 5"# };

        let on_first_line = multiline_str_lit.to_slice().slice(2..4);
        assert_eq!(on_first_line.text(), "ne");
        let err = on_first_line.error("Test error message").to_string();
        assert_eq!(
            err,
            r###"Test error message:
   --> /inside/a/test.rs:999:128
    |
999 | r#"Line 1
    |      ^^
"###
        );

        let on_middle_line = multiline_str_lit.to_slice().slice(9..=15);
        assert_eq!(on_middle_line.text(), "ne 2\nLi");
        let err = on_middle_line.error("Test error message").to_string();
        assert_eq!(
            err,
            r###"Test error message:
    --> /inside/a/test.rs:1000:3
     |
1000 |   Line 2
     |  ___^
1001 | | Line 3
     | |__^
"###
        );

        let on_middle_line = multiline_str_lit.to_slice().slice(9..=25);
        assert_eq!(on_middle_line.text(), "ne 2\nLine 3\nLine ");
        let err = on_middle_line.error("Test error message").to_string();
        assert_eq!(
            err,
            r###"Test error message:
    --> /inside/a/test.rs:1000:3
     |
1000 |   Line 2
     |  ___^
1001 | | Line 3
1002 | | Line 4
     | |_____^
"###
        );

        let on_last_line = multiline_str_lit.to_slice().slice(30..34);
        assert_eq!(on_last_line.text(), "ne 5");
        let err = on_last_line.error("Test error message").to_string();
        assert_eq!(
            err,
            r###"Test error message:
    --> /inside/a/test.rs:1003:3
     |
1003 | Line 5"#
     |   ^^^^
"###
        );

        let ends_on_line_break = multiline_str_lit.to_slice().slice(25..28);
        assert_eq!(ends_on_line_break.text(), " 4\n");
        let err = ends_on_line_break.error("Test error message").to_string();
        assert_eq!(
            err,
            r###"Test error message:
    --> /inside/a/test.rs:1002:5
     |
1002 | Line 4
     |     ^^^
"###
        );

        let ends_on_line_break_multi = multiline_str_lit.to_slice().slice(18..28);
        assert_eq!(ends_on_line_break_multi.text(), " 3\nLine 4\n");
        let err = ends_on_line_break_multi
            .error("Test error message")
            .to_string();
        assert_eq!(
            err,
            r###"Test error message:
    --> /inside/a/test.rs:1001:5
     |
1001 |   Line 3
     |  _____^
1002 | | Line 4
     | |_______^
"###
        );
    }

    #[test]
    fn test_error_fallback_truncated_multiline() {
        let multiline_str_lit = str_lit! { r#"Line 1
Line 2
Line 3
Line 4
Line 5
Line 6
Line 7
Line 8
Line 9"# };

        let on_first_line = multiline_str_lit.to_slice().slice(2..4);
        assert_eq!(on_first_line.text(), "ne");
        let err = on_first_line.error("Test error message").to_string();
        assert_eq!(
            err,
            r###"Test error message:
   --> /inside/a/test.rs:999:128
    |
999 | r#"Line 1
    |      ^^
"###
        );

        let on_middle_line = multiline_str_lit.to_slice().slice(9..=55);
        assert_eq!(
            on_middle_line.text(),
            "ne 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\n"
        );
        let err = on_middle_line.error("Test error message").to_string();
        assert_eq!(
            err,
            r###"Test error message:
    --> /inside/a/test.rs:1000:3
     |
1000 |   Line 2
     |  ___^
1001 | | Line 3
1002 | | Line 4
...    |
1005 | | Line 7
1006 | | Line 8
     | |_______^
"###
        );

        let on_last_line = multiline_str_lit.to_slice().slice(58..);
        assert_eq!(on_last_line.text(), "ne 9");
        let err = on_last_line.error("Test error message").to_string();
        assert_eq!(
            err,
            r###"Test error message:
    --> /inside/a/test.rs:1007:3
     |
1007 | Line 9"#
     |   ^^^^
"###
        );
    }
}
