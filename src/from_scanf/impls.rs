use super::FromScanf;

use std::{borrow::Cow, num::*, path::PathBuf};

use const_format::formatcp;

// float syntax: https://doc.rust-lang.org/std/primitive.f32.html#grammar
//
// Float  ::= Sign? ( 'inf' | 'infinity' | 'nan' | Number )
const FLOAT_REGEX: &str = formatcp!(r"{SIGN}?(?i:inf|infinity|nan|{NUMBER})",);
// Number ::= ( Digit+ | Digit+ '.' Digit* | Digit* '.' Digit+ ) Exp?
const NUMBER: &str = formatcp!(r"(?:{DIGIT}+|{DIGIT}+\.{DIGIT}*|{DIGIT}*\.{DIGIT}+)(?:{EXP})?",);
// Exp    ::= 'e' Sign? Digit+
const EXP: &str = formatcp!(r"e{SIGN}?{DIGIT}+");
// Sign   ::= [+-]
const SIGN: &str = r"[+-]";
// Digit  ::= [0-9]
const DIGIT: &str = r"\d";

macro_rules! doc_concat {
    ($target: item, $($doc: expr),+) => {
        $(
            #[doc = $doc]
        )+
        $target
    };
}

macro_rules! impl_num {
    ($spec: literal, $prefix: literal; $(($ty: ty, $n: literal),)+) => {
        impl_num!($spec, $prefix; $(($ty, $n, $n),)+);
    };
    ($spec: literal, $prefix: literal; $(($ty: ty, $n: literal, $doc: literal),)+) => {
        $(impl FromScanf<'_> for $ty {
            doc_concat!{
                const REGEX: &'static str = concat!($prefix, $n, "}");,
                "Matches ", $spec, " number with up to", stringify!($doc), "digits\n",
                "```",
                "# use sscanf::FromScanf; use std::num::*;",
                concat!("assert_eq!(", stringify!($ty), "::REGEX, r\"", $prefix, $n, "}\");"),
                "```"
            }

            fn from_match(input: &str) -> Option<Self> {
                input.parse().ok()
            }
        })+
    };
    (f64; $($ty: ty),+) => {
        $(impl FromScanf<'_> for $ty {
            doc_concat!{
                const REGEX: &'static str = FLOAT_REGEX;,
                "Matches any floating point number",
                "",
                concat!("See See [FromStr on ", stringify!($ty), "](https://doc.rust-lang.org/std/primitive.", stringify!($ty), ".html#method.from_str) for details"),
                "```",
                "# use sscanf::FromScanf;",
                concat!("assert_eq!(", stringify!($ty), r#"::REGEX, r"[+-]?(?i:inf|infinity|nan|(?:\d+|\d+\.\d*|\d*\.\d+)(?:e[+-]?\d+)?)");"#),
                "```"
            }

            fn from_match(input: &str) -> Option<Self> {
                input.parse().ok()
            }
        })+
    };
}

impl_num!("any positive", r"\+?\d{1,";
    (u8, 3),
    (u16, 5),
    (u32, 10),
    (u64, 20),
    (u128, 39),
);
impl_num!("any positive non-zero", r"\+?[1-9]\d{0,";
    (NonZeroU8, 2, 3),
    (NonZeroU16, 4, 5),
    (NonZeroU32, 9, 10),
    (NonZeroU64, 19, 20),
    (NonZeroU128, 38, 39),
);
impl_num!("any", r"[-+]?\d{1,";
    (i8, 3),
    (i16, 5),
    (i32, 10),
    (i64, 20),
    (i128, 39),
);
impl_num!("any non-zero", r"[-+]?[1-9]\d{0,";
    (NonZeroI8, 2, 3),
    (NonZeroI16, 4, 5),
    (NonZeroI32, 9, 10),
    (NonZeroI64, 19, 20),
    (NonZeroI128, 38, 39),
);
impl_num!(f64; f32, f64);

impl FromScanf<'_> for usize {
    /// Matches any positive integer.
    ///
    /// Note that other integer types are limited in how many digits they can match as a way to catch invalid input
    /// during the matching phase rather than the parsing phase. This is considerably faster than having a match
    /// succeed just to fail later on. However, `usize` depends on the platform, so we won't limit it at compile time
    /// and instead leave it to the runtime to fail if the input is too large.
    ///
    /// ```
    /// # use sscanf::FromScanf;
    /// assert_eq!(usize::REGEX, r"[-+]?\d+?")
    /// ```
    const REGEX: &'static str = r"[-+]?\d+?";

    fn from_match(input: &str) -> Option<Self> {
        input.parse().ok()
    }
}
impl FromScanf<'_> for isize {
    /// Matches any integer.
    ///
    /// See [`usize`](#impl-FromScanf<'_>-for-usize) for details.
    ///
    /// ```
    /// # use sscanf::FromScanf;
    /// assert_eq!(isize::REGEX, r"[-+]?\d+?")
    /// ```
    const REGEX: &'static str = usize::REGEX;

    fn from_match(input: &str) -> Option<Self> {
        input.parse().ok()
    }
}
impl FromScanf<'_> for NonZeroUsize {
    /// Matches any positive non-zero integer.
    ///
    /// See [`usize`](#impl-FromScanf<'_>-for-usize) for details.
    ///
    /// ```
    /// # use sscanf::FromScanf; use std::num::NonZeroUsize;
    /// assert_eq!(NonZeroUsize::REGEX, r"\+?[1-9]\d*?")
    /// ```
    const REGEX: &'static str = r"\+?[1-9]\d*?";

    fn from_match(input: &str) -> Option<Self> {
        input.parse().ok()
    }
}
impl FromScanf<'_> for NonZeroIsize {
    /// Matches any non-zero integer.
    ///
    /// See [`usize`](#impl-FromScanf<'_>-for-usize) for details.
    ///
    /// ```
    /// # use sscanf::FromScanf; use std::num::NonZeroIsize;
    /// assert_eq!(NonZeroIsize::REGEX, r"[-+]?[1-9]\d*?")
    /// ```
    const REGEX: &'static str = r"[-+]?[1-9]\d*?";

    fn from_match(input: &str) -> Option<Self> {
        input.parse().ok()
    }
}

impl FromScanf<'_> for String {
    /// Matches any sequence of Characters.
    ///
    /// Note that this clones part of the input string, which is usually not necessary.
    /// Use [`str`](#impl-FromScanf<'_>-for-%26str) unless you explicitly need ownership.
    /// ```
    /// # use sscanf::FromScanf;
    /// assert_eq!(String::REGEX, r".+?")
    /// ```
    const REGEX: &'static str = r".+?";

    fn from_match(input: &str) -> Option<Self> {
        Some(input.to_string())
    }
}
impl<'input> FromScanf<'input> for &'input str {
    /// Matches any sequence of Characters.
    ///
    /// This is one of the few types that borrow part of the input string, so you need to keep lifetimes in mind when
    /// using this type. If the input string doesn't live long enough, use [`String`](#impl-FromScanf<'_>-for-String)
    /// instead.
    /// ```
    /// # use sscanf::FromScanf;
    /// assert_eq!(<&str>::REGEX, r".+?")
    /// ```
    const REGEX: &'static str = String::REGEX;

    fn from_match(input: &'input str) -> Option<Self> {
        Some(input)
    }
}
impl<'input> FromScanf<'input> for Cow<'input, str> {
    /// Matches any sequence of Characters.
    ///
    /// This is one of the few types that borrow part of the input string, so you need to keep lifetimes in mind when
    /// using this type. If the input string doesn't live long enough, use [`String`](#impl-FromScanf<'_>-for-String)
    /// instead.
    /// ```
    /// # use sscanf::FromScanf; use std::borrow::Cow;
    /// assert_eq!(Cow::<str>::REGEX, r".+?")
    /// ```
    const REGEX: &'static str = String::REGEX;

    fn from_match(input: &'input str) -> Option<Self> {
        Some(Cow::Borrowed(input))
    }
}
impl FromScanf<'_> for char {
    /// Matches a single Character.
    /// ```
    /// # use sscanf::FromScanf;
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
impl FromScanf<'_> for bool {
    /// Matches `true` or `false`.
    /// ```
    /// # use sscanf::FromScanf;
    /// assert_eq!(bool::REGEX, r"true|false")
    /// ```
    const REGEX: &'static str = r"true|false";

    fn from_match(input: &str) -> Option<Self> {
        input.parse().ok()
    }
}

impl FromScanf<'_> for PathBuf {
    /// Matches any sequence of Characters.
    ///
    /// Paths in `std` don't actually have any restrictions on what they can contain, so anything
    /// is valid.
    /// ```
    /// # use sscanf::FromScanf; use std::path::PathBuf;
    /// assert_eq!(PathBuf::REGEX, r".+?")
    /// ```
    const REGEX: &'static str = String::REGEX;

    fn from_match(input: &str) -> Option<Self> {
        input.parse().ok()
    }
}

#[test]
#[rustfmt::skip]
fn no_capture_groups() {
    macro_rules! check {
        ($($ty: ty),+) => {
            $(
                let regex = regex_automata::meta::Regex::new(<$ty>::REGEX).unwrap();
                assert_eq!(regex.captures_len(), 1, "Regex for {} >>{}<< has capture groups", stringify!($ty), <$ty>::REGEX);
                // 1 for the whole match
            )+ 
        };
    }

    check!(u8, u16, u32, u64, u128, usize);
    check!(i8, i16, i32, i64, i128, isize);
    check!(NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize);
    check!(NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize);
    check!(f32, f64);
    check!(String, &str, Cow<str>, char, bool);
    check!(PathBuf);
}
