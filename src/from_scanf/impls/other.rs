use std::{borrow::Cow, path::PathBuf};

use crate::{advanced::*, *};

/// Matches any sequence of characters.
///
/// Note that this clones part of the input string, which is usually not necessary.
/// Use [`&str`](#impl-FromScanf<'input>-for-%26str) unless you explicitly need ownership.
impl FromScanf<'_> for String {
    fn get_matcher(_: &FormatOptions) -> Matcher {
        Matcher::from_regex(r".+?").unwrap()
    }

    fn from_match(matches: Match<'_, '_>, _: &FormatOptions) -> Option<Self> {
        Some(matches.text().to_string())
    }
}
impl AcceptsRegexOverride<'_> for String {
    fn from_regex_match(input: &str, _: &FormatOptions) -> Option<Self> {
        Some(input.to_string())
    }
}

/// Matches any sequence of characters.
///
/// This is one of the few types that borrow part of the input string, so you need to keep lifetimes in mind when
/// using this type. If the input string doesn't live long enough, use [`String`](#impl-FromScanf<'_>-for-String) instead.
impl<'input> FromScanf<'input> for &'input str {
    fn get_matcher(_: &FormatOptions) -> Matcher {
        Matcher::from_regex(r".+?").unwrap()
    }

    fn from_match(matches: Match<'_, 'input>, _: &FormatOptions) -> Option<Self> {
        Some(matches.text())
    }
}
impl<'input> AcceptsRegexOverride<'input> for &'input str {
    fn from_regex_match(input: &'input str, _: &FormatOptions) -> Option<Self> {
        Some(input)
    }
}

/// Matches any sequence of characters.
///
/// This is one of the few types that borrow part of the input string, so you need to keep lifetimes in mind when
/// using this type. If the input string doesn't live long enough, use [`String`](#impl-FromScanf<'_>-for-String) instead.
impl<'input> FromScanf<'input> for Cow<'input, str> {
    fn get_matcher(_: &FormatOptions) -> Matcher {
        Matcher::from_regex(r".+?").unwrap()
    }

    fn from_match(matches: Match<'_, 'input>, _: &FormatOptions) -> Option<Self> {
        Some(Cow::Borrowed(matches.text()))
    }
}
impl<'input> AcceptsRegexOverride<'input> for Cow<'input, str> {
    fn from_regex_match(input: &'input str, _: &FormatOptions) -> Option<Self> {
        Some(Cow::Borrowed(input))
    }
}

/// Matches a single character.
impl FromScanf<'_> for char {
    fn get_matcher(_: &FormatOptions) -> Matcher {
        Matcher::from_regex(r".").unwrap()
    }

    fn from_match(matches: Match<'_, '_>, _: &FormatOptions) -> Option<Self> {
        parse_char(matches.text())
    }
}
impl AcceptsRegexOverride<'_> for char {
    fn from_regex_match(input: &str, _: &FormatOptions) -> Option<Self> {
        parse_char(input)
    }
}
fn parse_char(input: &str) -> Option<char> {
    let mut iter = input.chars();
    let ret = iter.next()?;
    if iter.next().is_some() {
        return None;
    }
    Some(ret)
}

/// Matches common representations of boolean values.
///
/// True values: `true`, `1`, `yes`, `on`, `t` (all case-insensitive)\
/// False values: `false`, `0`, `no`, `off`, `f` (all case-insensitive)
///
/// Feel free to open an issue if you have other suggestions for common boolean representations.
impl FromScanf<'_> for bool {
    fn get_matcher(_: &FormatOptions) -> Matcher {
        Matcher::from_regex(r"(?i:true|false|1|0|yes|no|on|off|t|f)").unwrap()
    }

    fn from_match(matches: Match<'_, '_>, _: &FormatOptions) -> Option<Self> {
        parse_bool(matches.text())
    }
}
impl AcceptsRegexOverride<'_> for bool {
    fn from_regex_match(input: &str, _: &FormatOptions) -> Option<Self> {
        parse_bool(input)
    }
}
fn parse_bool(input: &str) -> Option<bool> {
    match input.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" | "t" => Some(true),
        "false" | "0" | "no" | "off" | "f" => Some(false),
        _ => None,
    }
}

/// Matches any sequence of characters.
///
/// Paths in `std` don't actually have any restrictions on what they can contain, so anything is valid.\
/// If you need more specific parsing (e.g. standard unix paths), consider using a regex override.
impl FromScanf<'_> for PathBuf {
    fn get_matcher(_: &FormatOptions) -> Matcher {
        Matcher::from_regex(r".+?").unwrap()
    }

    fn from_match(matches: Match<'_, '_>, _: &FormatOptions) -> Option<Self> {
        matches.text().parse().ok()
    }
}
impl AcceptsRegexOverride<'_> for PathBuf {
    fn from_regex_match(input: &str, _: &FormatOptions) -> Option<Self> {
        input.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // a test to see if it is possible to implement FromScanf for a generic tuple

    impl<'input, A, B> FromScanf<'input> for (A, B)
    where
        A: FromScanf<'input>,
        B: FromScanf<'input>,
    {
        fn get_matcher(options: &FormatOptions) -> Matcher {
            Matcher::Seq(vec![
                MatchPart::literal("("),
                A::get_matcher(options).into(),
                MatchPart::regex(r",\s*").unwrap(),
                B::get_matcher(options).into(),
                MatchPart::literal(")"),
            ])
        }

        fn from_match(matches: Match<'_, 'input>, options: &FormatOptions) -> Option<Self> {
            let matches = matches.as_seq();
            let a = matches.parse_at(1, options)?;
            let b = matches.parse_at(3, options)?;
            Some((a, b))
        }
    }

    #[test]
    fn test_tuple_parser() {
        let input = "(1, 2)";
        type Tuple = (u8, u8);
        let output = parse::<Tuple>(input);
        assert_eq!(output.unwrap(), (1, 2));
    }
}
