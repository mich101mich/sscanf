use std::borrow::Cow;

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
    /// let re = Option::<bool>::create_parser(&Default::default()).0.regex();
    /// assert_eq!(re, r"((true|false)?)");
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
    fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
        let (inner_re, inner) = SubType::<T>::new(format);
        let re = RegexSegment::from_known(format!("{inner_re}?"), inner_re.num_capture_groups());
        (re, OptionParser(inner))
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

pub struct ProvidedFromScanfParser;

impl<'input> FromScanf<'input> for &'input str {
    type Parser = ProvidedFromScanfParser;

    /// Matches any sequence of Characters.
    /// ```
    /// # use sscanf::FromScanf;
    /// let re = <&str>::create_parser(&Default::default()).0.regex();
    /// assert_eq!(re, r"(.+?)");
    ///
    /// let string_re = String::create_parser(&Default::default()).0.regex();
    /// assert_eq!(re, string_re);
    /// ```
    fn create_parser(_: &FormatOptions) -> (RegexSegment, Self::Parser) {
        (RegexSegment::from_known(r".+?", 0), ProvidedFromScanfParser)
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
    /// # use sscanf::FromScanf; use std::borrow::Cow;
    /// let re = Cow::<str>::create_parser(&Default::default()).0.regex();
    /// assert_eq!(re, r"(.+?)");
    ///
    /// let string_re = String::create_parser(&Default::default()).0.regex();
    /// assert_eq!(re, string_re);
    /// ```
    fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
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
    /// let re = String::create_parser(&Default::default()).0.regex();
    /// assert_eq!(re, r"(.+?)");
    ///
    /// let str_re = <&str>::create_parser(&Default::default()).0.regex();
    /// assert_eq!(re, str_re);
    /// ```
    fn create_parser(_: &FormatOptions) -> (RegexSegment, Self::Parser) {
        (RegexSegment::from_known(r".+?", 0), ())
    }
}

impl FromScanf<'_> for char {
    type Parser = ();

    /// Matches a single Character.
    /// ```
    /// # use sscanf::FromScanf;
    /// let re = char::create_parser(&Default::default()).0.regex();
    /// assert_eq!(re, r"(.)");
    /// ```
    fn create_parser(_: &FormatOptions) -> (RegexSegment, Self::Parser) {
        (RegexSegment::from_known(r".", 0), ())
    }
}

impl FromScanf<'_> for bool {
    type Parser = ();

    /// Matches `true` or `false`.
    /// ```
    /// # use sscanf::FromScanf;
    /// let re = bool::create_parser(&Default::default()).0.regex();
    /// assert_eq!(re, r"(true|false)");
    /// ```
    fn create_parser(_: &FormatOptions) -> (RegexSegment, Self::Parser) {
        (RegexSegment::from_known(r"true|false", 0), ())
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
    /// let re = PathBuf::create_parser(&Default::default()).0.regex();
    /// assert_eq!(re, r"(.+?)");
    /// ```
    fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
        String::create_parser(format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // a test to see if it is possible to implement FromScanf for a tuple

    impl<'input, A, B> FromScanf<'input> for (A, B)
    where
        A: FromScanf<'input>,
        B: FromScanf<'input>,
    {
        type Parser = TupleParser<'input, A, B>;

        fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
            let (a_regex, a) = SubType::<A>::new(format);
            let (b_regex, b) = SubType::<B>::new(format);
            let regex = RegexSegment::from_known(
                format!(r"\({a_regex},\s*{b_regex}\)"),
                a_regex.num_capture_groups() + b_regex.num_capture_groups(),
            );
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
