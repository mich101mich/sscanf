use std::num::*;

use super::*;

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

        let mut needs_case_insensitive = false;

        let prefix = format
            .prefix()
            .map(|prefix| {
                needs_case_insensitive = true; // the prefix contains letters
                match format.prefix_policy() {
                    NumberPrefixPolicy::Forbidden => unreachable!(),
                    NumberPrefixPolicy::Required => prefix.to_string(),
                    NumberPrefixPolicy::Optional => format!("(?:{prefix})?"),
                }
            })
            .unwrap_or_default();

        // possible characters for digits
        let possible_chars = if radix <= 10 {
            format!("0-{}", radix - 1)
        } else {
            needs_case_insensitive = true; // letters are now involved
            let last_letter = (b'a' + radix as u8 - 1 - 10) as char;
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
        // sign prefix ( [ possible_chars ] { 1, num_digits } )
        // - "sign" matches an optional sign, either '+' or '-'
        // - "prefix" matches the optional prefix, if any
        // - "(...)" creates a capture group for the number itself
        //   - "[possible_chars]" matches one of the possible characters for the given radix
        //   - "{1,num_digits}" means repeat the previous character class one to `num_digits` times
        let mut regex = format!("{sign}{prefix}([{possible_chars}]{{1,{num_digits}}})");

        if needs_case_insensitive {
            // "(?i:...)" a non-capturing group that makes the inside case-insensitive.
            regex = format!("(?i:{regex})");
        }

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

macro_rules! impl_int {
    ($( $unsigned:ty | $signed:ty : ( $digits_2:literal , $digits_10:literal , $digits_16:literal ) ),+) => {
        $(
            impl PrimitiveNumber for $unsigned {
                const BITS: u32 = <$unsigned>::BITS;
                const SIGNED: bool = false;
                fn parse_radix(s: &str, radix: u32) -> Option<Self> {
                    <$unsigned>::from_str_radix(s, radix).ok()
                }

                type Unsigned = $unsigned;
                fn negative_from_unsigned(_: Self::Unsigned) -> Option<Self> {
                    None // unsigned types cannot be negative, so this is always None
                }
            }

            impl FromScanf<'_> for $unsigned {
                type Parser = NumberParser<$unsigned>;

                doc_concat!{
                    fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
                        NumberParser::<$unsigned>::new(format.number)
                    },
                    concat!("Matches an unsigned number with ", $digits_2, " bits in the respective radix."),
                    "",
                    "```",
                    "# use sscanf::*; use sscanf::NumberFormatOption::*; use sscanf::NumberPrefixPolicy::*;",
                    concat!("let re = ", stringify!($unsigned), "::create_parser(&Default::default()).0.regex();"),
                    concat!(r#"assert_eq!(re, r"((?i:\+?([0-9]{1,"#, $digits_10, r#"})))");"#),
                    "",
                    "let hex_options = FormatOptions::builder().hex().with_prefix().build();",
                    concat!("let re = ", stringify!($unsigned), "::create_parser(&hex_options).0.regex();"),
                    concat!(r#"assert_eq!(re, r"((?i:\+?0x([0-9a-f]{1,"#, $digits_16, r#"})))");"#),
                    "```"
                }
            }

            impl PrimitiveNumber for $signed {
                const BITS: u32 = <$signed>::BITS;
                const SIGNED: bool = true;
                fn parse_radix(s: &str, radix: u32) -> Option<Self> {
                    <$signed>::from_str_radix(s, radix).ok()
                }

                type Unsigned = $unsigned;
                fn negative_from_unsigned(n: Self::Unsigned) -> Option<Self> {
                    // The simplest way to convert an unsigned number to its signed negative counterpart is to
                    // calculate `0 - n` with the method below.
                    <$signed>::checked_sub_unsigned(0, n)
                }
            }

            impl FromScanf<'_> for $signed {
                type Parser = NumberParser<$signed>;

                doc_concat!{
                    fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
                        NumberParser::<$signed>::new(format.number)
                    },
                    concat!("Matches a signed number with ", $digits_2, " bits in the respective radix."),
                    "",
                    "```",
                    "# use sscanf::*; use sscanf::NumberFormatOption::*; use sscanf::NumberPrefixPolicy::*;",
                    concat!("let re = ", stringify!($signed), "::create_parser(&Default::default()).0.regex();"),
                    concat!(r#"assert_eq!(re, r"((?i:[-+]?([0-9]{1,"#, $digits_10, r#"})))");"#),
                    "",
                    "let hex_options = FormatOptions::builder().hex().with_prefix().build();",
                    concat!("let re = ", stringify!($signed), "::create_parser(&hex_options).0.regex();"),
                    concat!(r#"assert_eq!(re, r"((?i:[-+]?0x([0-9a-f]{1,"#, $digits_16, r#"})))");"#),
                    "```"
                }
            }
        )+
    };
}
impl_int!(u8|i8: (8, 3, 2), u16|i16: (16, 5, 4), u32|i32: (32, 10, 8), u64|i64: (64, 20, 16), u128|i128: (128, 39, 32));

impl PrimitiveNumber for usize {
    const BITS: u32 = <usize>::BITS;
    const SIGNED: bool = false;
    fn parse_radix(s: &str, radix: u32) -> Option<Self> {
        usize::from_str_radix(s, radix).ok()
    }

    type Unsigned = usize;
    fn negative_from_unsigned(_: Self::Unsigned) -> Option<Self> {
        None // unsigned types cannot be negative, so this is always None
    }
}

impl FromScanf<'_> for usize {
    type Parser = NumberParser<usize>;

    /// Matches an unsigned number with a platform-specific number of bits in the respective radix.
    ///
    /// ```
    /// # use sscanf::*; use sscanf::NumberFormatOption::*; use sscanf::NumberPrefixPolicy::*;
    /// #[cfg(target_pointer_width = "64")]
    /// {
    ///     let re = usize::create_parser(&Default::default()).0.regex();
    ///     assert_eq!(re, r"((?i:\+?([0-9]{1,20})))");
    ///
    ///     let hex_options = FormatOptions::builder().hex().with_prefix().build();
    ///     let re = usize::create_parser(&hex_options).0.regex();
    ///     assert_eq!(re, r"((?i:\+?0x([0-9a-f]{1,16})))");
    /// }
    /// #[cfg(target_pointer_width = "32")]
    /// {
    ///     let re = usize::create_parser(&Default::default()).0.regex();
    ///     assert_eq!(re, r"((?i:\+?([0-9]{1,10})))");
    ///
    ///     let hex_options = FormatOptions::builder().hex().with_prefix().build();
    ///     let re = usize::create_parser(&hex_options).0.regex();
    ///     assert_eq!(re, r"((?i:\+?0x([0-9a-f]{1,8})))");
    /// }
    /// ```
    fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
        NumberParser::<usize>::new(format.number)
    }
}

impl PrimitiveNumber for isize {
    const BITS: u32 = <isize>::BITS;
    const SIGNED: bool = true;
    fn parse_radix(s: &str, radix: u32) -> Option<Self> {
        isize::from_str_radix(s, radix).ok()
    }

    type Unsigned = usize;
    fn negative_from_unsigned(n: Self::Unsigned) -> Option<Self> {
        // The simplest way to convert an unsigned number to its signed negative counterpart is to
        // calculate `0 - n` with the method below.
        isize::checked_sub_unsigned(0, n)
    }
}

impl FromScanf<'_> for isize {
    type Parser = NumberParser<isize>;

    /// Matches a signed number with a platform-specific number of bits in the respective radix.
    ///
    /// ```
    /// # use sscanf::*; use sscanf::NumberFormatOption::*; use sscanf::NumberPrefixPolicy::*;
    /// #[cfg(target_pointer_width = "64")]
    /// {
    ///     let re = isize::create_parser(&Default::default()).0.regex();
    ///     assert_eq!(re, r"((?i:[-+]?([0-9]{1,20})))");
    ///
    ///     let hex_options = FormatOptions::builder().hex().with_prefix().build();
    ///     let re = isize::create_parser(&hex_options).0.regex();
    ///     assert_eq!(re, r"((?i:[-+]?0x([0-9a-f]{1,16})))");
    /// }
    /// #[cfg(target_pointer_width = "32")]
    /// {
    ///     let re = isize::create_parser(&Default::default()).0.regex();
    ///     assert_eq!(re, r"((?i:[-+]?([0-9]{1,10})))");
    ///
    ///     let hex_options = FormatOptions::builder().hex().with_prefix().build();
    ///     let re = isize::create_parser(&hex_options).0.regex();
    ///     assert_eq!(re, r"((?i:[-+]?0x([0-9a-f]{1,8})))");
    /// }
    /// ```
    fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
        NumberParser::<isize>::new(format.number)
    }
}

macro_rules! impl_non_zero {
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
                doc_concat!{
                    fn create_parser(format: &FormatOptions) -> (RegexSegment, Self::Parser) {
                        NumberParser::<$ty>::new(format.number)
                    },
                    concat!("Matches a non-zero [", stringify!($base), "](trait.FromScanf.html#impl-FromScanf<'_>-for-", stringify!($base), ").")
                }
            }
        )+
    };
}
impl_non_zero!(NonZeroU8:u8, NonZeroU16:u16, NonZeroU32:u32, NonZeroU64:u64, NonZeroU128:u128, NonZeroUsize:usize);
impl_non_zero!(NonZeroI8:i8, NonZeroI16:i16, NonZeroI32:i32, NonZeroI64:i64, NonZeroI128:i128, NonZeroIsize:isize);

macro_rules! impl_float {
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
                concat!("let re = ", stringify!($ty), "::create_parser(&Default::default()).0.regex();"),
                r#"assert_eq!(re, r"([+-]?(?i:inf|infinity|nan|(?:\d+|\d+\.\d*|\d*\.\d+)(?:e[+-]?\d+)?))");"#,
                "```"
            }
        })+
    };
}
impl_float!(f64; f32, f64);

#[cfg(test)]
mod tests {
    use super::*;

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

    macro_rules! create_type_limit_tests {
        ($ty:ty) => {
            use NumberFormatOption::*;
            use NumberPrefixPolicy::*;
            let max_str = <$ty>::MAX.to_string();
            let min_str = <$ty>::MIN.to_string();
            assert_parse::<$ty>(<$ty>::MAX, &max_str, Default::default());
            assert_parse::<$ty>(<$ty>::MIN, &min_str, Default::default());
            assert_parse::<$ty>(0, "0", Default::default());
            assert_parse::<$ty>(35, "35", Default::default());
            assert_parse::<$ty>(5, "5", Hexadecimal(Forbidden));
            assert_parse::<$ty>(10, "a", Hexadecimal(Forbidden));
            assert_parse::<$ty>(10, "A", Hexadecimal(Forbidden));
            assert_parse::<$ty>(15, "f", Hexadecimal(Forbidden));

            assert_parse::<$ty>(15, "f", Hexadecimal(Forbidden));
            assert_parse_fails::<$ty>("0xf", Hexadecimal(Forbidden));
            assert_parse_fails::<$ty>("0Xf", Hexadecimal(Forbidden));

            assert_parse::<$ty>(15, "f", Hexadecimal(Optional));
            assert_parse::<$ty>(15, "0xf", Hexadecimal(Optional));
            assert_parse::<$ty>(15, "0Xf", Hexadecimal(Optional));

            assert_parse_fails::<$ty>("f", Hexadecimal(Required));
            assert_parse::<$ty>(15, "0xf", Hexadecimal(Required));
            assert_parse::<$ty>(15, "0Xf", Hexadecimal(Required));

            let too_much = str_add_one(&max_str, 10);
            assert_parse_fails::<$ty>(&too_much, Default::default());

            let max_hex = format!("{:x}", <$ty>::MAX);
            let max_hex_prefixed = format!("0x{max_hex}");
            assert_parse::<$ty>(<$ty>::MAX, &max_hex, Hexadecimal(Forbidden));
            assert_parse::<$ty>(<$ty>::MAX, &max_hex, Hexadecimal(Optional));
            assert_parse_fails::<$ty>(&max_hex, Hexadecimal(Required));

            assert_parse_fails::<$ty>(&max_hex_prefixed, Hexadecimal(Forbidden));
            assert_parse::<$ty>(<$ty>::MAX, &max_hex_prefixed, Hexadecimal(Optional));
            assert_parse::<$ty>(<$ty>::MAX, &max_hex_prefixed, Hexadecimal(Required));

            let too_much_hex = str_add_one(&max_hex, 16);
            assert_parse_fails::<$ty>(&too_much_hex, Hexadecimal(Forbidden));
        };
    }

    macro_rules! create_signed_type_limit_tests {
        ($ty:ty) => {
            create_type_limit_tests!($ty);

            let min_str = <$ty>::MIN.to_string();
            let too_little = str_add_one(&min_str, 10);
            assert_parse_fails::<$ty>(&too_little, Default::default());

            let raw_min_hex = format!("{:x}", <$ty>::MIN.abs_diff(0));
            let min_hex = format!("-{raw_min_hex}");
            assert_parse::<$ty>(<$ty>::MIN, &min_hex, Hexadecimal(Forbidden));
            assert_parse::<$ty>(<$ty>::MIN, &min_hex, Hexadecimal(Optional));
            assert_parse_fails::<$ty>(&min_hex, Hexadecimal(Required));

            let min_hex_prefixed = format!("-0x{raw_min_hex}");
            assert_parse_fails::<$ty>(&min_hex_prefixed, Hexadecimal(Forbidden));
            assert_parse::<$ty>(<$ty>::MIN, &min_hex_prefixed, Hexadecimal(Optional));
            assert_parse::<$ty>(<$ty>::MIN, &min_hex_prefixed, Hexadecimal(Required));

            let too_little_hex = str_add_one(&min_hex, 16);
            assert_parse_fails::<$ty>(&too_little_hex, Hexadecimal(Forbidden));
        };
    }

    #[rustfmt::skip]
    mod test_spam {
        use super::*;
        // test functions need to have unique names, but we can't create identifiers with macros yet, so
        // we have to manually write these out.
        #[test] fn test_type_parser_limits_u8() { create_type_limit_tests!(u8); }
        #[test] fn test_type_parser_limits_u16() { create_type_limit_tests!(u16); }
        #[test] fn test_type_parser_limits_u32() { create_type_limit_tests!(u32); }
        #[test] fn test_type_parser_limits_u64() { create_type_limit_tests!(u64); }
        #[test] fn test_type_parser_limits_u128() { create_type_limit_tests!(u128); }
        #[test] fn test_type_parser_limits_usize() { create_type_limit_tests!(usize); }
        #[test] fn test_type_parser_limits_i8() { create_signed_type_limit_tests!(i8); }
        #[test] fn test_type_parser_limits_i16() { create_signed_type_limit_tests!(i16); }
        #[test] fn test_type_parser_limits_i32() { create_signed_type_limit_tests!(i32); }
        #[test] fn test_type_parser_limits_i64() { create_signed_type_limit_tests!(i64); }
        #[test] fn test_type_parser_limits_i128() { create_signed_type_limit_tests!(i128); }
        #[test] fn test_type_parser_limits_isize() { create_signed_type_limit_tests!(isize); }
    }

    macro_rules! create_float_parser_test {
        ($ty:ty, $ident:ident) => {
            assert_parse::<$ty>(1.0, "1.0", Default::default());
            assert_parse::<$ty>(-1.0, "-1.0", Default::default());
            assert_parse::<$ty>(
                std::$ident::consts::PI,
                &std::$ident::consts::PI.to_string(),
                Default::default(),
            );
            assert_parse::<$ty>(
                -std::$ident::consts::PI,
                &(-std::$ident::consts::PI).to_string(),
                Default::default(),
            );
            assert_parse::<$ty>(10_000_000_000.0, "1e10", Default::default());
            assert_parse::<$ty>(-10_000_000_000.0, "-1e10", Default::default());
            assert_parse::<$ty>(0.000_000_000_1, "1e-10", Default::default());
            assert_parse::<$ty>(-0.000_000_000_1, "-1e-10", Default::default());

            // Infinity and NaN
            assert_parse::<$ty>(<$ty>::INFINITY, "inf", Default::default());
            assert_parse::<$ty>(<$ty>::INFINITY, "infinity", Default::default());
            // not checking NaN here, because the assert_eq would fail

            // Case insensitivity
            assert_parse::<$ty>(<$ty>::INFINITY, "INF", Default::default());
            assert_parse::<$ty>(<$ty>::INFINITY, "INFINITY", Default::default());
            assert_parse::<$ty>(<$ty>::INFINITY, "iNfiNIty", Default::default());
        };
    }
    #[test]
    fn test_float_parser_f32() {
        create_float_parser_test!(f32, f32);
    }
    #[test]
    fn test_float_parser_f64() {
        create_float_parser_test!(f64, f64);
    }
}
