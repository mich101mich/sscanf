use std::{borrow::Cow, num::*};

use super::*;

impl<'input, T> FromScanf<'input> for Option<T>
where
    T: FromScanf<'input>,
{
    type Parser = OptionParser<'input, T>;

    /// Matches the inner type optionally.
    ///
    /// ```
    /// # use sscanf::FromScanf;
    /// assert_eq!(Option::<bool>::regex().0, r"(true|false)?")
    /// ```
    ///
    /// Note that `Option<T>` can be used to match large optional blocks of text by placing a `(...)?` around it:
    /// ```
    /// # use sscanf::*;
    /// let input1 = "There might be text and a number here: text says 0";
    /// let input2 = "There might be text and a number here:"; // no text
    ///
    /// let result1 = sscanf!(input1, r"There might be text and a number here:( text says {})?", Option<usize>).unwrap();
    /// let result2 = sscanf!(input2, r"There might be text and a number here:( text says {})?", Option<usize>).unwrap();
    /// // note the use of raw format string (r"") to allow regex syntax in the format string
    ///
    /// assert_eq!(result1, Some(0));
    /// assert_eq!(result2, None);
    /// ```
    ///
    /// Though this bears the risk of the outer text being there but the inner text not:
    /// ```
    /// # use sscanf::*;
    /// let input = "There might be text and a number here: text says "; // all but the number
    ///
    /// let result = sscanf!(input, r"There might be text and a number here:( text says {})?", Option<usize>).unwrap();
    /// // logically, you would expect this to fail, since the number is missing, but that is the nature of optional matches
    ///
    /// assert_eq!(result, None);
    /// ```
    fn create_parser(mut format: FormatOptions) -> (RegexSegment, Self::Parser) {
        let custom_regex = format.regex.take();
        let (inner_regex, inner) = SubType::<T>::new(format);
        let mut regex = RegexSegment::from_known(
            &format!("{}?", inner_regex),
            inner_regex.num_capture_groups(),
        );
        regex._maybe_replace_with::<Self>(custom_regex);
        (regex, OptionParser(inner))
    }
}

pub struct OptionParser<'input, T: FromScanf<'input>>(SubType<'input, T>);

impl<'input, T: FromScanf<'input>> FromScanfParser<'input, Option<T>> for OptionParser<'input, T> {
    /// Parses the inner type if it is present, otherwise returns `None`.
    fn parse(&self, _: &'input str, mut sub_matches: &[Option<&'input str>]) -> Option<Option<T>> {
        Some(if sub_matches[0].is_some() {
            Some(self.0.parse(&mut sub_matches)?)
        } else {
            None
        })
    }
}

// float syntax: https://doc.rust-lang.org/std/primitive.f32.html#grammar
//
// Float  ::= Sign? ( 'inf' | 'infinity' | 'nan' | Number )
// Number ::= ( Digit+ | Digit+ '.' Digit* | Digit* '.' Digit+ ) Exp?
// Exp    ::= 'e' Sign? Digit+
// Sign   ::= [+-]
// Digit  ::= [0-9]
// Float  ::= Sign? ( 'inf' | 'infinity' | 'nan' | Number )
const FLOAT: &str = r"[+-]?(?i:inf|infinity|nan|(?:\d+|\d+\.\d*|\d*\.\d+)(?:e[+-]?\d+)?)";

macro_rules! doc_concat {
    ($target: item, $($doc: expr),+) => {
        $(#[doc = $doc])+
        $target
    };
}

// TODO: number format parsing

macro_rules! impl_num {
    ($spec: literal, $prefix: literal; $(($ty: ty, $n: literal)),+) => {
        impl_num!($spec, $prefix; $(($ty, $n, $n)),+);
    };

    ($spec: literal, $prefix: literal; $(($ty: ty, $n: literal, $doc: literal)),+) => {
        $(impl FromScanf<'_> for $ty {
            type Parser = ();

            doc_concat!{
                fn create_parser(format: FormatOptions) -> (RegexSegment, ()) {
                    RegexSegment::from_known(concat!($prefix, $n, "}"), 0)._with_format::<Self, _>(format)
                },
                "Matches ", $spec, " number with up to", stringify!($doc), "digits\n",
                "```",
                "# use sscanf::FromScanf; use std::num::*;",
                concat!("assert_eq!(", stringify!($ty), "::regex().0, r\"", $prefix, $n, "}\");"),
                "```"
            }
        })+
    };

    (f64; $($ty: ty),+) => {
        $(impl FromScanf<'_> for $ty {
            type Parser = ();

            doc_concat!{
                fn create_parser(format: FormatOptions) -> (RegexSegment, ()) {
                    RegexSegment::from_known(FLOAT, 0)._with_format::<Self, _>(format)
                },
                "Matches any floating point number",
                "",
                concat!("See See [FromStr on ", stringify!($ty), "](https://doc.rust-lang.org/std/primitive.", stringify!($ty), ".html#method.from_str) for details"),
                "```",
                "# use sscanf::FromScanf;",
                concat!("assert_eq!(", stringify!($ty), r#"::regex().0, r"[+-]?(?i:inf|infinity|nan|(?:\d+|\d+\.\d*|\d*\.\d+)(?:e[+-]?\d+)?)");"#),
                "```"
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
    (usize, 20)
);
impl_num!("any positive non-zero", r"\+?[1-9]\d{0,";
    (NonZeroU8, 2, 3),
    (NonZeroU16, 4, 5),
    (NonZeroU32, 9, 10),
    (NonZeroU64, 19, 20),
    (NonZeroU128, 38, 39),
    (NonZeroUsize, 19, 20)
);
impl_num!("any", r"[-+]?\d{1,";
    (i8, 3),
    (i16, 5),
    (i32, 10),
    (i64, 20),
    (i128, 39),
    (isize, 20)
);
impl_num!("any non-zero", r"[-+]?[1-9]\d{0,";
    (NonZeroI8, 2, 3),
    (NonZeroI16, 4, 5),
    (NonZeroI32, 9, 10),
    (NonZeroI64, 19, 20),
    (NonZeroI128, 38, 39),
    (NonZeroIsize, 19, 20)
);
impl_num!(f64; f32, f64);

#[derive(Default)]
pub struct ProvidedFromScanfParser;

impl<'input> FromScanf<'input> for &'input str {
    type Parser = ProvidedFromScanfParser;

    /// Matches any sequence of Characters.
    /// ```
    /// # use sscanf::FromScanf;
    /// assert_eq!(<&str>::regex().0, r".+?")
    /// assert_eq!(<&str>::regex(), String::regex())
    /// ```
    fn create_parser(format: FormatOptions) -> (RegexSegment, Self::Parser) {
        RegexSegment::from_known(r".+?", 0)._with_format::<Self, _>(format)
    }
}
impl<'input> FromScanfParser<'input, &'input str> for ProvidedFromScanfParser {
    /// Returns the full match as a reference to the input string.
    fn parse(&self, full_match: &'input str, _: &[Option<&'input str>]) -> Option<&'input str> {
        Some(full_match)
    }
}

impl<'input> FromScanf<'input> for Cow<'input, str> {
    type Parser = ProvidedFromScanfParser;

    /// Matches any sequence of Characters.
    ///
    /// ```
    /// # use sscanf::FromScanf;
    /// assert_eq!(Cow::<str>::regex().0, r".+?")
    /// assert_eq!(Cow::<str>::regex(), String::regex())
    /// ```
    fn create_parser(format: FormatOptions) -> (RegexSegment, Self::Parser) {
        <&str>::create_parser(format)
    }
}
impl<'input> FromScanfParser<'input, Cow<'input, str>> for ProvidedFromScanfParser {
    /// Returns a `Cow::Borrowed` of the full match.
    fn parse(
        &self,
        full_match: &'input str,
        _: &[Option<&'input str>],
    ) -> Option<Cow<'input, str>> {
        Some(Cow::Borrowed(full_match))
    }
}

impl FromScanf<'_> for String {
    type Parser = ();

    /// Matches any sequence of Characters.
    /// ```
    /// # use sscanf::FromScanf;
    /// assert_eq!(String::regex().0, r".+?")
    /// assert_eq!(String::regex(), <&str>::regex())
    /// ```
    fn create_parser(format: FormatOptions) -> (RegexSegment, Self::Parser) {
        RegexSegment::from_known(r".+?", 0)._with_format::<Self, _>(format)
    }
}

impl FromScanf<'_> for char {
    type Parser = ();

    /// Matches a single Character.
    /// ```
    /// # use sscanf::FromScanf;
    /// assert_eq!(char::regex().0, r".")
    /// ```
    fn create_parser(format: FormatOptions) -> (RegexSegment, Self::Parser) {
        RegexSegment::from_known(r".", 0)._with_format::<Self, _>(format)
    }
}

impl FromScanf<'_> for bool {
    type Parser = ();

    /// Matches `true` or `false`.
    /// ```
    /// # use sscanf::FromScanf;
    /// assert_eq!(bool::regex().0, r"true|false")
    /// ```
    fn create_parser(format: FormatOptions) -> (RegexSegment, Self::Parser) {
        RegexSegment::from_known(r"true|false", 0)._with_format::<Self, _>(format)
    }
}

impl FromScanf<'_> for std::path::PathBuf {
    type Parser = ();

    /// Matches any sequence of Characters.
    ///
    /// Paths in `std` don't actually have any restrictions on what they can contain, so anything
    /// is valid.
    /// ```
    /// # use sscanf::FromScanf; use std::path::PathBuf;
    /// assert_eq!(PathBuf::regex().0, r".+")
    /// ```
    fn create_parser(format: FormatOptions) -> (RegexSegment, Self::Parser) {
        String::create_parser(format)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    impl<'input, A, B> FromScanf<'input> for (A, B)
    where
        A: FromScanf<'input>,
        B: FromScanf<'input>,
    {
        type Parser = TupleParser<'input, A, B>;

        fn create_parser(mut format: FormatOptions) -> (RegexSegment, Self::Parser) {
            let custom_regex = format.regex.take();
            let (a_regex, a) = SubType::<A>::new(format.clone()); // clone is fine, because the only non-copy component is the regex, which we took out
            let (b_regex, b) = SubType::<B>::new(format);
            let mut regex = RegexSegment::from_known(
                &format!(r"\({},\s*{}\)", a_regex, b_regex),
                a_regex.num_capture_groups() + b_regex.num_capture_groups(),
            );
            regex._maybe_replace_with::<Self>(custom_regex);
            (regex, TupleParser { a, b })
        }
    }
    pub struct TupleParser<'input, A: FromScanf<'input>, B: FromScanf<'input>> {
        a: SubType<'input, A>,
        b: SubType<'input, B>,
    }
    impl<'input, A: FromScanf<'input>, B: FromScanf<'input>> FromScanfParser<'input, (A, B)>
        for TupleParser<'input, A, B>
    {
        fn parse(&self, _: &'input str, mut sub_matches: &[Option<&'input str>]) -> Option<(A, B)> {
            let a = self.a.parse(&mut sub_matches)?;
            let b = self.b.parse(&mut sub_matches)?;
            Some((a, b))
        }
    }
}
