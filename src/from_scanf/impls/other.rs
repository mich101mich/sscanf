use std::{borrow::Cow, path::PathBuf};

use crate::*;

impl FromScanfSimple<'_> for String {
    /// Matches any sequence of Characters.
    ///
    /// Note that this clones part of the input string, which is usually not necessary.
    /// Use [`str`](#impl-FromScanfSimple<'_>-for-%26str) unless you explicitly need ownership.
    /// ```
    /// # use sscanf::FromScanfSimple;
    /// assert_eq!(String::REGEX, r".+?")
    /// ```
    const REGEX: &'static str = r".+?";

    fn from_match(input: &str) -> Option<Self> {
        Some(input.to_string())
    }
}
impl<'input> FromScanfSimple<'input> for &'input str {
    /// Matches any sequence of Characters.
    ///
    /// This is one of the few types that borrow part of the input string, so you need to keep lifetimes in mind when
    /// using this type. If the input string doesn't live long enough, use [`String`](#impl-FromScanfSimple<'_>-for-String)
    /// instead.
    /// ```
    /// # use sscanf::FromScanfSimple;
    /// assert_eq!(<&str>::REGEX, r".+?")
    /// ```
    const REGEX: &'static str = String::REGEX;

    fn from_match(input: &'input str) -> Option<Self> {
        Some(input)
    }
}
impl<'input> FromScanfSimple<'input> for Cow<'input, str> {
    /// Matches any sequence of Characters.
    ///
    /// This is one of the few types that borrow part of the input string, so you need to keep lifetimes in mind when
    /// using this type. If the input string doesn't live long enough, use [`String`](#impl-FromScanfSimple<'_>-for-String)
    /// instead.
    /// ```
    /// # use sscanf::FromScanfSimple; use std::borrow::Cow;
    /// assert_eq!(Cow::<str>::REGEX, r".+?")
    /// ```
    const REGEX: &'static str = String::REGEX;

    fn from_match(input: &'input str) -> Option<Self> {
        Some(Cow::Borrowed(input))
    }
}
impl FromScanfSimple<'_> for char {
    /// Matches a single Character.
    /// ```
    /// # use sscanf::FromScanfSimple;
    /// assert_eq!(char::REGEX, r".")
    /// ```
    const REGEX: &'static str = r".";

    fn from_match(input: &str) -> Option<Self> {
        let mut iter = input.chars();
        let ret = iter.next()?;
        if iter.next().is_some() {
            return None; // more than one character
        }
        Some(ret)
    }
}
impl FromScanfSimple<'_> for bool {
    /// Matches `true` or `false`.
    /// ```
    /// # use sscanf::FromScanfSimple;
    /// assert_eq!(bool::REGEX, r"true|false")
    /// ```
    const REGEX: &'static str = r"true|false";

    fn from_match(input: &str) -> Option<Self> {
        input.parse().ok()
    }
}

impl FromScanfSimple<'_> for PathBuf {
    /// Matches any sequence of Characters.
    ///
    /// Paths in `std` don't actually have any restrictions on what they can contain, so anything
    /// is valid.
    /// ```
    /// # use sscanf::FromScanfSimple; use std::path::PathBuf;
    /// assert_eq!(PathBuf::REGEX, r".+?")
    /// ```
    const REGEX: &'static str = String::REGEX;

    fn from_match(input: &str) -> Option<Self> {
        input.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::advanced::*;

    // a test to see if it is possible to implement FromScanf for a generic tuple

    impl<'input, A, B> FromScanf<'input> for (A, B)
    where
        A: FromScanf<'input>,
        B: FromScanf<'input>,
    {
        fn get_matcher(format: &FormatOptions) -> Matcher {
            Matcher::Seq(vec![
                MatchPart::literal("("),
                A::get_matcher(format).into(),
                MatchPart::regex(r",\s*"),
                B::get_matcher(format).into(),
                MatchPart::literal(")"),
            ])
        }

        fn from_match_tree(matches: MatchTree<'_, 'input>, format: &FormatOptions) -> Option<Self> {
            let matches = matches.as_seq();
            let a = matches.parse_at(0, format)?;
            let b = matches.parse_at(1, format)?;
            Some((a, b))
        }
    }

    #[test]
    fn test_tuple_parser() {
        let input = "(1, 2)";
        let parser = __macro_utilities::Parser::new();
        type Tuple = (u8, u8);
        parser.assert_compiled(|| Tuple::get_matcher(&Default::default()));
        let output = parser.parse_captures(input, |matches| {
            Tuple::from_match_tree(matches, &Default::default())
        });
        assert_eq!(output.unwrap(), (1, 2));
    }
}
