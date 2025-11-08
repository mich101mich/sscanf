use std::num::*;

use crate::{
    advanced::{format_options::*, *},
    *,
};

// float syntax: https://doc.rust-lang.org/std/primitive.f32.html#grammar
//
// Float  ::= Sign? ( 'inf' | 'infinity' | 'nan' | Number )
// Number ::= ( Digit+ | Digit+ '.' Digit* | Digit* '.' Digit+ ) Exp?
// Exp    ::= 'e' Sign? Digit+
// Sign   ::= [+-]
// Digit  ::= [0-9]
// Float  ::= Sign? ( 'inf' | 'infinity' | 'nan' | Number )
const FLOAT: &str =
    r"[+-]?(?i:inf|infinity|nan|(?:[0-9]+|[0-9]+\.[0-9]*|[0-9]*\.[0-9]+)(?:e[+-]?[0-9]+)?)";

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

fn primitive_get_matcher<T: PrimitiveNumber>(format: &FormatOptions) -> Matcher {
    let format = &format.number;
    let radix = format.to_number();

    use regex_syntax::hir::*;
    fn optional(inner: Hir) -> Hir {
        // regex_syntax uses a repetition for optional groups
        Hir::repetition(Repetition {
            min: 0,
            max: Some(1),
            greedy: true,
            sub: Box::new(inner),
        })
    }
    fn cl(c: char) -> ClassUnicodeRange {
        ClassUnicodeRange::new(c, c)
    }

    // The final regex is:
    // sign prefix ( [ possible_chars ] { 1, num_digits } )
    // - "sign" matches an optional sign, either '+' or '-'
    // - "prefix" matches the prefix, if any
    // - "(...)" creates a capture group for the number itself
    //   - "[possible_chars]" matches one of the possible characters for the given radix
    //   - "{1,num_digits}" means repeat the previous character class one to `num_digits` times
    let mut regex = vec![];

    regex.push(optional(if T::SIGNED {
        Hir::class(Class::Unicode(ClassUnicode::new([cl('-'), cl('+')]))) // "[-+]?"
    } else {
        Hir::literal(b"+".as_slice()) // "\\+?"
    }));

    fn make_prefix(lower: char) -> Hir {
        let zero = Hir::literal(b"0".as_slice());

        let upper = lower.to_ascii_uppercase();
        let specifier = Hir::class(Class::Unicode(ClassUnicode::new([cl(lower), cl(upper)])));

        Hir::concat(vec![zero, specifier])
    }
    fn add_prefix(regex: &mut Vec<Hir>, format: NumberFormatOption) {
        let (lower, policy) = match format {
            NumberFormatOption::Binary(number_prefix_policy) => ('b', number_prefix_policy),
            NumberFormatOption::Octal(number_prefix_policy) => ('o', number_prefix_policy),
            NumberFormatOption::Decimal => return,
            NumberFormatOption::Hexadecimal(number_prefix_policy) => ('x', number_prefix_policy),
            NumberFormatOption::Other(_) => return,
        };
        match policy {
            NumberPrefixPolicy::Forbidden => {}
            NumberPrefixPolicy::Optional => regex.push(optional(make_prefix(lower))),
            NumberPrefixPolicy::Required => regex.push(make_prefix(lower)),
        }
    }
    add_prefix(&mut regex, *format);

    // possible characters for digits
    let possible_chars = if radix <= 10 {
        // "0-{radix - 1}"
        let range = ClassUnicodeRange::new('0', (b'0' + radix as u8 - 1) as char);
        Hir::class(Class::Unicode(ClassUnicode::new([range])))
    } else {
        // "0-9a-{'a' + radix - 1 - 10}"
        let number_range = ClassUnicodeRange::new('0', '9');
        let last_letter_offset = radix as u8 - 1 - 10;
        let letter_range = ClassUnicodeRange::new('a', (b'a' + last_letter_offset) as char);
        let upper_letter_range = ClassUnicodeRange::new('A', (b'A' + last_letter_offset) as char);
        Hir::class(Class::Unicode(ClassUnicode::new([
            number_range,
            letter_range,
            upper_letter_range,
        ])))
    };

    let num_digits = if radix == 2 {
        T::BITS
    } else {
        // digit conversion:   num_digits_in_base_a = num_digits_in_base_b * log(b) / log(a)
        // where log can be any type of logarithm. Since binary is base 2 and log_2(2) = 1,
        // we can use log_2 to simplify the math
        f32::ceil(T::BITS as f32 / f32::log2(radix as f32)) as u32
    };

    let digit_matcher = Hir::repetition(Repetition {
        min: 1,
        max: Some(num_digits),
        greedy: true,
        sub: Box::new(possible_chars),
    });
    let digit_matcher = Hir::capture(Capture {
        index: 0, // will be overwritten by matcher compilation
        name: None,
        sub: Box::new(digit_matcher),
    });
    regex.push(digit_matcher);

    Matcher::from_inner(MatcherType::Raw(Hir::concat(regex)))
}

fn primitive_from_match_tree<T: PrimitiveNumber>(
    matches: MatchTree<'_, '_>,
    format: &FormatOptions,
) -> Option<T> {
    let number = if matches.num_children() != 1 {
        // User specified a custom regex override. Assume that the entire match is the number.
        matches.text()
    } else {
        matches.at(0).text()
    };
    if matches.text().starts_with('-') {
        // negative numbers have a different range from positive numbers (e.g. i8::MIN is -128 while i8::MAX is 127).
        // in order to avoid an overflow when trying to parse number like -128i8, we need to parse the number as its
        // unsigned counterpart, e.g. u8::parse_radix("128", 10). This is better than having to manually check for
        // the overflowing value or constructing a new string with a leading minus sign.
        let raw_num = T::Unsigned::parse_radix(number, format.number.to_number())?;
        T::negative_from_unsigned(raw_num)
    } else {
        T::parse_radix(number, format.number.to_number())
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
                fn get_matcher(format: &FormatOptions) -> Matcher {
                    primitive_get_matcher::<$unsigned>(format)
                }
                doc_concat!{
                    fn from_match_tree(matches: MatchTree<'_, '_>, format: &FormatOptions) -> Option<Self> {
                        primitive_from_match_tree::<$unsigned>(matches, format)
                    },
                    concat!("Matches an unsigned number with ", $digits_2, " bits in the respective radix."),
                    "",
                    "```",
                    "# use sscanf::*; use sscanf::advanced::*;",
                    concat!("let re = ", stringify!($unsigned), "::get_matcher(&Default::default()).to_regex();"),
                    concat!(r#"assert_eq!(re, r"((?:\+?([0-9]{1,"#, $digits_10, r#"})))");"#),
                    "",
                    "let hex_options = FormatOptions::builder().hex().with_prefix().build();",
                    concat!("let re = ", stringify!($unsigned), "::get_matcher(&hex_options).to_regex();"),
                    concat!(r#"assert_eq!(re, r"((?:\+?0[Xx]([0-9A-Fa-f]{1,"#, $digits_16, r#"})))");"#),
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
                fn get_matcher(format: &FormatOptions) -> Matcher {
                    primitive_get_matcher::<$signed>(format)
                }

                doc_concat!{
                    fn from_match_tree(matches: MatchTree<'_, '_>, format: &FormatOptions) -> Option<Self> {
                        primitive_from_match_tree::<$signed>(matches, format)
                    },
                    concat!("Matches a signed number with ", $digits_2, " bits in the respective radix."),
                    "",
                    "```",
                    "# use sscanf::*; use sscanf::advanced::*;",
                    concat!("let re = ", stringify!($signed), "::get_matcher(&Default::default()).to_regex();"),
                    concat!(r#"assert_eq!(re, r"((?:[\+\-]?([0-9]{1,"#, $digits_10, r#"})))");"#),
                    "",
                    "let hex_options = FormatOptions::builder().hex().with_prefix().build();",
                    concat!("let re = ", stringify!($signed), "::get_matcher(&hex_options).to_regex();"),
                    concat!(r#"assert_eq!(re, r"((?:[\+\-]?0[Xx]([0-9A-Fa-f]{1,"#, $digits_16, r#"})))");"#),
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
    fn get_matcher(format: &FormatOptions) -> Matcher {
        primitive_get_matcher::<usize>(format)
    }

    /// Matches an unsigned number with a platform-specific number of bits in the respective radix.
    ///
    /// ```
    /// # use sscanf::*; use sscanf::advanced::*;
    /// #[cfg(target_pointer_width = "64")]
    /// {
    ///     let re = usize::get_matcher(&Default::default()).to_regex();
    ///     assert_eq!(re, r"((?:\+?([0-9]{1,20})))");
    ///
    ///     let hex_options = FormatOptions::builder().hex().with_prefix().build();
    ///     let re = usize::get_matcher(&hex_options).to_regex();
    ///     assert_eq!(re, r"((?:\+?0[Xx]([0-9A-Fa-f]{1,16})))");
    /// }
    /// #[cfg(target_pointer_width = "32")]
    /// {
    ///     let re = usize::get_matcher(&Default::default()).to_regex();
    ///     assert_eq!(re, r"((?:\+?([0-9]{1,10})))");
    ///
    ///     let hex_options = FormatOptions::builder().hex().with_prefix().build();
    ///     let re = usize::get_matcher(&hex_options).to_regex();
    ///     assert_eq!(re, r"((?:\+?0[Xx]([0-9A-Fa-f]{1,8})))");
    /// }
    /// ```
    fn from_match_tree(matches: MatchTree<'_, '_>, format: &FormatOptions) -> Option<Self> {
        primitive_from_match_tree::<usize>(matches, format)
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
    fn get_matcher(format: &FormatOptions) -> Matcher {
        primitive_get_matcher::<isize>(format)
    }

    /// Matches a signed number with a platform-specific number of bits in the respective radix.
    ///
    /// ```
    /// # use sscanf::*; use sscanf::advanced::*;
    /// #[cfg(target_pointer_width = "64")]
    /// {
    ///     let re = isize::get_matcher(&Default::default()).to_regex();
    ///     assert_eq!(re, r"((?:[\+\-]?([0-9]{1,20})))");
    ///
    ///     let hex_options = FormatOptions::builder().hex().with_prefix().build();
    ///     let re = isize::get_matcher(&hex_options).to_regex();
    ///     assert_eq!(re, r"((?:[\+\-]?0[Xx]([0-9A-Fa-f]{1,16})))");
    /// }
    /// #[cfg(target_pointer_width = "32")]
    /// {
    ///     let re = isize::get_matcher(&Default::default()).to_regex();
    ///     assert_eq!(re, r"((?:[\+\-]?([0-9]{1,10})))");
    ///
    ///     let hex_options = FormatOptions::builder().hex().with_prefix().build();
    ///     let re = isize::get_matcher(&hex_options).to_regex();
    ///     assert_eq!(re, r"((?:[\+\-]?0[Xx]([0-9A-Fa-f]{1,8})))");
    /// }
    /// ```
    fn from_match_tree(matches: MatchTree<'_, '_>, format: &FormatOptions) -> Option<Self> {
        primitive_from_match_tree::<isize>(matches, format)
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
                fn get_matcher(format: &FormatOptions) -> Matcher {
                    primitive_get_matcher::<$base>(format)
                }
                doc_concat!{
                    fn from_match_tree(matches: MatchTree<'_, '_>, format: &FormatOptions) -> Option<Self> {
                        primitive_from_match_tree::<$base>(matches, format).and_then(Self::new)
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
    ($($ty: ty),+) => {
        $(impl FromScanfSimple<'_> for $ty {
            const REGEX: &'static str = FLOAT;

            doc_concat!{
                fn from_match(input: &str) -> Option<Self> {
                    input.parse().ok()
                },
                "Matches any floating point number",
                "",
                concat!("See [FromStr on ", stringify!($ty), "](https://doc.rust-lang.org/std/primitive.", stringify!($ty), ".html#method.from_str) for details"),
                "```",
                "# use sscanf::FromScanfSimple;",
                concat!("let re = ", stringify!($ty), "::REGEX;"),
                r#"assert_eq!(re, r"[+-]?(?i:inf|infinity|nan|(?:[0-9]+|[0-9]+\.[0-9]*|[0-9]*\.[0-9]+)(?:e[+-]?[0-9]+)?)");"#,
                "```"
            }
        })+
    };
}
impl_float!(f32, f64);

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

        let format = FormatOptions {
            number: options,
            ..Default::default()
        };
        let parser = __macro_utilities::Parser::new();
        parser.assert_compiled(|| T::get_matcher(&format));
        let output =
            parser.parse_captures(value_str, |matches| T::from_match_tree(matches, &format));

        let Some(parsed_value) = output else {
            panic!(
                "Matcher {:?} does not match {value_str} for {name}",
                T::get_matcher(&format)
            );
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

        let format = FormatOptions {
            number: options,
            ..Default::default()
        };
        let parser = __macro_utilities::Parser::new();
        parser.assert_compiled(|| T::get_matcher(&format));
        let result =
            parser.parse_captures(value_str, |matches| T::from_match_tree(matches, &format));

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
