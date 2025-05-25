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
    /// assert_eq!(Option::<bool>::create_parser(&Default::default()).0.regex(), r"((true|false)?)");
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

trait PrimitiveNumber: Sized {
    /// The number of bits in the type.
    const BITS: u32;
    /// Whether the type is signed or not.
    const SIGNED: bool;
    /// Parses a number from a string in the given radix.
    ///
    /// Effectively `from_str_radix`, but calling it that would create naming conflicts.
    fn parse_radix(s: &str, radix: u32) -> Option<Self>;

    /// The unsigned counterpart of the type (or the type itself if it is unsigned).
    type Unsigned: PrimitiveNumber;
    /// Converts an unsigned number to a negative signed number, if possible.
    fn negative_from_unsigned(n: Self::Unsigned) -> Option<Self>;
}

pub struct NumberParser<T> {
    radix: u32,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> NumberParser<T> {
    fn new(format: NumberFormatOption) -> (RegexSegment, Self)
    where
        T: PrimitiveNumber,
    {
        let radix = format.to_number();

        let sign = if T::SIGNED { "[-+]?" } else { "\\+?" };

        let prefix = format
            .prefix()
            .map(|prefix| match format.prefix_policy() {
                NumberPrefixPolicy::Forbidden => unreachable!(),
                NumberPrefixPolicy::Required => prefix.to_string(),
                NumberPrefixPolicy::Optional => format!("(?:{prefix})?"),
            })
            .unwrap_or_default();

        // possible characters for digits
        let possible_chars = if radix <= 10 {
            format!("0-{}", radix - 1)
        } else {
            let last_letter = (b'a' + radix as u8 - 10) as char;
            format!("0-9a-{last_letter}")
        };

        let num_digits = if radix == 2 {
            T::BITS
        } else {
            // digit conversion:   num_digits_in_base_a = num_digits_in_base_b * log(b) / log(a)
            // where log can be any type of logarithm. Since binary is base 2 and log_2(2) = 1,
            // we can use log_2 to simplify the math
            f32::ceil(T::BITS as f32 / f32::log2(radix as f32)) as u32
        };

        // The final regex is:
        // (?i) sign prefix ( [ possible_chars ] { 1, num_digits } ) (?-i)
        // - "(?i)" Makes the capture group case-insensitive
        // - "sign" matches an optional sign, either '+' or '-'
        // - "prefix" matches the optional prefix, if any
        // - "(...)" creates a capture group for the number itself
        //   - "[possible_chars]" matches one of the possible characters for the given radix
        //   - "{1,num_digits}" means repeat the previous character class one to `num_digits` times
        // - "(?-i)" Ends the case-insensitive mode
        let regex = format!("(?i){sign}{prefix}([{possible_chars}]{{1,{num_digits}}})(?-i)");
        let regex = RegexSegment::from_known(regex, 1);

        let parser = NumberParser {
            radix,
            _phantom: std::marker::PhantomData,
        };
        (regex, parser)
    }
}

impl<'input, T: PrimitiveNumber> FromScanfParser<'input, T> for NumberParser<T> {
    fn parse(&self, full_match: &'input str, sub_matches: &[Option<&'input str>]) -> Option<T> {
        let number = sub_matches[0].unwrap();
        if full_match.starts_with('-') {
            // negative numbers have a different range from positive numbers (e.g. i8::MIN is -128 while i8::MAX is 127).
            // in order to avoid an overflow when trying to parse number like -128i8, we need to parse the number as its
            // unsigned counterpart, e.g. u8::parse_radix("128", 10). This is better than having to manually check for
            // the overflowing value or constructing a new string with a leading minus sign.
            let raw_num = T::Unsigned::parse_radix(number, self.radix)?;
            T::negative_from_unsigned(raw_num)
        } else {
            T::parse_radix(number, self.radix)
        }
    }
}

macro_rules! impl_num {
    ($( $ty:ty : $digits:literal ),+) => {
        $(
            impl PrimitiveNumber for $ty {
                const BITS: u32 = <$ty>::BITS;
                const SIGNED: bool = false;
                fn parse_radix(s: &str, radix: u32) -> Option<Self> {
                    <$ty>::from_str_radix(s, radix).ok()
                }

                type Unsigned = $ty;
                fn negative_from_unsigned(_: Self::Unsigned) -> Option<Self> {
                    None // unsigned types cannot be negative, so this is always None
                }
            }

            impl FromScanf<'_> for $ty {
                type Parser = NumberParser<$ty>;

                doc_concat!{
                    fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
                        NumberParser::<$ty>::new(format.number)
                    },
                    "Matches stuff",
                    "",
                    "```",
                    "# use sscanf::FromScanf;",
                    concat!(
                        "assert_eq!(",stringify!($ty), "::create_parser(&Default::default()).0.regex(),",
                        r##" r"((?i)\+?([0-9]{1,"##, $digits, r##"})(?-i))");"##,
                    ),
                    "```"
                }
            }
        )+
    };
}

macro_rules! impl_num_signed {
    ($( $ty:ty : $unsigned:ty ),+) => {
        $(
            impl PrimitiveNumber for $ty {
                const BITS: u32 = <$ty>::BITS;
                const SIGNED: bool = true;
                fn parse_radix(s: &str, radix: u32) -> Option<Self> {
                    <$ty>::from_str_radix(s, radix).ok()
                }

                type Unsigned = $unsigned;
                fn negative_from_unsigned(n: Self::Unsigned) -> Option<Self> {
                    // The simplest way to convert an unsigned number to its signed negative counterpart is to
                    // calculate `0 - n` with the method below.
                    <$ty>::checked_sub_unsigned(0, n)
                }
            }

            impl FromScanf<'_> for $ty {
                type Parser = NumberParser<$ty>;
                fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
                    NumberParser::<$ty>::new(format.number)
                }
            }
        )+
    };
}

macro_rules! impl_num_nz {
    ($( $ty:ty : $base:ty ),+) => {
        $(
            impl PrimitiveNumber for $ty {
                const BITS: u32 = <$ty>::BITS;
                const SIGNED: bool = <$base>::SIGNED;
                fn parse_radix(s: &str, radix: u32) -> Option<Self> {
                    <$base>::from_str_radix(s, radix).ok().and_then(Self::new)
                }

                type Unsigned = <$base as PrimitiveNumber>::Unsigned;
                fn negative_from_unsigned(n: Self::Unsigned) -> Option<Self> {
                    <$base>::negative_from_unsigned(n).and_then(Self::new)
                }
            }

            impl FromScanf<'_> for $ty {
                type Parser = NumberParser<$ty>;
                fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
                    NumberParser::<$ty>::new(format.number)
                }
            }
        )+
    };
}

macro_rules! impl_num_float {
    (f64; $($ty: ty),+) => {
        $(impl FromScanf<'_> for $ty {
            type Parser = ();

            doc_concat!{
                fn create_parser(_: &FormatOptions) -> (RegexSegment, ()) {
                    (RegexSegment::from_known(FLOAT, 0), ())
                },
                "Matches any floating point number",
                "",
                concat!("See [FromStr on ", stringify!($ty), "](https://doc.rust-lang.org/std/primitive.", stringify!($ty), ".html#method.from_str) for details"),
                "```",
                "# use sscanf::FromScanf;",
                concat!("assert_eq!(", stringify!($ty), r#"::create_parser(&Default::default()).0.regex(), r"([+-]?(?i:inf|infinity|nan|(?:\d+|\d+\.\d*|\d*\.\d+)(?:e[+-]?\d+)?))");"#),
                "```"
            }
        })+
    };
}

impl_num!(u8:3, u16:5, u32:10, u64:20, u128:39, usize:20);
impl_num_signed!(i8:u8, i16:u16, i32:u32, i64:u64, i128:u128, isize:usize);
impl_num_nz!(NonZeroU8:u8, NonZeroU16:u16, NonZeroU32:u32, NonZeroU64:u64, NonZeroU128:u128, NonZeroUsize:usize);
impl_num_nz!(NonZeroI8:i8, NonZeroI16:i16, NonZeroI32:i32, NonZeroI64:i64, NonZeroI128:i128, NonZeroIsize:isize);
impl_num_float!(f64; f32, f64);

pub struct ProvidedFromScanfParser;

impl<'input> FromScanf<'input> for &'input str {
    type Parser = ProvidedFromScanfParser;

    /// Matches any sequence of Characters.
    /// ```
    /// # use sscanf::FromScanf;
    /// assert_eq!(<&str>::create_parser(&Default::default()).0.regex(), r"(.+?)");
    /// assert_eq!(<&str>::create_parser(&Default::default()).0, String::create_parser(&Default::default()).0);
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
    /// assert_eq!(Cow::<str>::create_parser(&Default::default()).0.regex(), r"(.+?)");
    /// assert_eq!(Cow::<str>::create_parser(&Default::default()).0, String::create_parser(&Default::default()).0);
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
    /// assert_eq!(String::create_parser(&Default::default()).0.regex(), r"(.+?)");
    /// assert_eq!(String::create_parser(&Default::default()).0, <&str>::create_parser(&Default::default()).0);
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
    /// assert_eq!(char::create_parser(&Default::default()).0.regex(), r"(.)");
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
    /// assert_eq!(bool::create_parser(&Default::default()).0.regex(), r"(true|false)");
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
    /// assert_eq!(PathBuf::create_parser(&Default::default()).0.regex(), r"(.+?)");
    /// ```
    fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
        String::create_parser(format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn to_string_radix(digit: i32) -> char {
        "0123456789abcdefghijklmnopqrstuvwxyz".as_bytes()[digit as usize] as char
    }
    /// Adds one to a string representation of a number in the given radix.
    ///
    /// Has to be done like this because we want to exceed the maximum value of all types, including u128 and i128.
    fn str_add_one(mut s: &str, radix: u32) -> String {
        let negative = if let Some(stripped) = s.strip_prefix('-') {
            s = stripped;
            true
        } else {
            false
        };
        let bytes = s.as_bytes();
        let mut carry = 1;
        let mut result = Vec::with_capacity(bytes.len());
        for &byte in bytes.iter().rev() {
            let mut digit = (byte as char).to_digit(radix).unwrap() as i32;
            digit += carry;
            carry = 0;
            if digit >= radix as i32 {
                digit -= radix as i32;
                carry = 1;
            } else if digit < 0 {
                digit += radix as i32;
                carry = -1;
            }
            result.push(to_string_radix(digit));
        }
        if carry > 0 {
            result.push(to_string_radix(carry));
        }
        if negative {
            result.push('-');
        }
        result.reverse();
        result.into_iter().collect()
    }

    #[track_caller]
    fn assert_parse<T>(value: T, value_str: &str, options: NumberFormatOption)
    where
        T: ToString + PartialEq + std::fmt::Debug + for<'input> FromScanf<'input>,
    {
        let name = std::any::type_name::<T>();

        let (regex, parser) = T::create_parser(&FormatOptions {
            number: options,
            ..Default::default()
        });
        let Ok(re) = regex::Regex::new(&format!("^{}$", regex.regex())) else {
            panic!("Failed to create regex for {name}: {}", regex.regex());
        };
        assert_eq!(
            re.captures_len(),
            regex.num_capture_groups() + 1,
            "Regex {re:?} has wrong number of capture groups for {name}",
        );

        let Some(captures) = re.captures(value_str) else {
            panic!("Regex {re:?} does not match {value_str} for {name}");
        };

        let mut iter = captures.iter().skip(1).map(|m| m.map(|m| m.as_str()));
        let full_match = iter.next().unwrap().unwrap();
        let sub_matches: Vec<_> = iter.collect();
        let Some(parsed_value) = parser.parse(full_match, &sub_matches) else {
            panic!("Parser for {name} failed to parse {value_str} ({value:?})");
        };
        assert_eq!(
            parsed_value, value,
            "Parser for {name} did not parse {value_str} correctly",
        );
    }

    #[track_caller]
    fn assert_parse_fails<T>(value_str: &str, options: NumberFormatOption)
    where
        T: ToString + PartialEq + std::fmt::Debug + for<'input> FromScanf<'input>,
    {
        let name = std::any::type_name::<T>();

        let (regex, parser) = T::create_parser(&FormatOptions {
            number: options,
            ..Default::default()
        });
        let Ok(re) = regex::Regex::new(&format!("^{}$", regex.regex())) else {
            panic!("Failed to create regex for {name}: {}", regex.regex());
        };
        assert_eq!(
            re.captures_len(),
            regex.num_capture_groups() + 1,
            "Regex {re:?} has wrong number of capture groups for {name}",
        );

        let Some(captures) = re.captures(value_str) else {
            return; // Expected matching failure
        };

        let mut iter = captures.iter().skip(1).map(|m| m.map(|m| m.as_str()));
        let full_match = iter.next().unwrap().unwrap();
        let sub_matches: Vec<_> = iter.collect();
        let result = parser.parse(full_match, &sub_matches);
        assert!(
            result.is_none(),
            "Parser for {name} should have failed to parse {value_str}, but it succeeded with {result:?}",
        );
    }

    macro_rules! create_signed_type_limit_tests {
        ( $( $ty:ty ),+ ) => {
            $({
                use NumberFormatOption::*;
                use NumberPrefixPolicy::*;
                let max_str = <$ty>::MAX.to_string();
                let min_str = <$ty>::MIN.to_string();
                assert_parse::<$ty>(<$ty>::MAX, &max_str, Default::default());
                assert_parse::<$ty>(<$ty>::MIN, &min_str, Default::default());
                assert_parse::<$ty>(0, "0", Default::default());

                let too_much = str_add_one(&max_str, 10);
                let too_little = str_add_one(&min_str, 10);
                assert_parse_fails::<$ty>(&too_much, Default::default());
                assert_parse_fails::<$ty>(&too_little, Default::default());

                let max_hex = format!("{:x}", <$ty>::MAX);
                let max_hex_prefixed = format!("0x{max_hex}");
                assert_parse::<$ty>(<$ty>::MAX, &max_hex, Hexadecimal(Forbidden));
                assert_parse::<$ty>(<$ty>::MAX, &max_hex, Hexadecimal(Optional));
                assert_parse_fails::<$ty>(&max_hex, Hexadecimal(Required));

                assert_parse_fails::<$ty>(&max_hex_prefixed, Hexadecimal(Forbidden));
                assert_parse::<$ty>(<$ty>::MAX, &max_hex_prefixed, Hexadecimal(Optional));
                assert_parse::<$ty>(<$ty>::MAX, &max_hex_prefixed, Hexadecimal(Required));

                let raw_min_hex = format!("{:x}", <$ty>::MIN.abs_diff(0));
                let min_hex = format!("-{raw_min_hex}");
                let min_hex_prefixed = format!("-0x{raw_min_hex}");
                assert_parse::<$ty>(<$ty>::MIN, &min_hex, Hexadecimal(Forbidden));
                assert_parse::<$ty>(<$ty>::MIN, &min_hex, Hexadecimal(Optional));
                assert_parse_fails::<$ty>(&min_hex, Hexadecimal(Required));

                assert_parse_fails::<$ty>(&min_hex_prefixed, Hexadecimal(Forbidden));
                assert_parse::<$ty>(<$ty>::MIN, &min_hex_prefixed, Hexadecimal(Optional));
                assert_parse::<$ty>(<$ty>::MIN, &min_hex_prefixed, Hexadecimal(Required));

                let too_much_hex = str_add_one(&max_hex, 16);
                let too_little_hex = str_add_one(&min_hex, 16);
                assert_parse_fails::<$ty>(&too_much_hex, Hexadecimal(Forbidden));
                assert_parse_fails::<$ty>(&too_little_hex, Hexadecimal(Forbidden));
            })+
        };
    }

    #[test]
    fn test_type_parser_limits() {
        create_signed_type_limit_tests!(i8, i16, i32, i64, i128, isize);
    }
}
