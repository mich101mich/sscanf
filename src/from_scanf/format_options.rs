use std::borrow::Cow;

use super::*;

/// The possible values for the format option in a [`FromScanf`] implementation.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FormatOptions {
    /// A overwrite with a custom regex (e.g. `{:/[a-d]+/}`).
    pub regex: Option<RegexSegment>,
    /// A formatter for a number (e.g. `{:x}`). If not present, defaults to base 10.
    pub number: NumberFormatOption,
    /// A fully custom format string (e.g. `{:[%Y-%m-%dT%H:%M:%SZ]}` for a chrono timestamp).
    ///
    /// Normally just a `&'static str` borrowing from the format string. Defined as a [`Cow`] just in case some custom
    /// implementations needs to pass a custom-custom format string to a subtype.
    pub custom: Option<Cow<'static, str>>,
}

/// The possible number formats for a number formatter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumberFormatOption {
    /// A binary number using `{:b}`, `{:B}`, `{:#b}`, or `{:#B}`. Prefix (if allowed) is `0b` or `0B`.
    Binary(NumberPrefixPolicy),
    /// An octal number using `{:o}`, `{:O}`, `{:#o}`, or `{:#O}`. Prefix (if allowed) is `0o` or `0O`.
    Octal(NumberPrefixPolicy),
    /// A decimal number. No prefix allowed. The default number format.
    #[default]
    Decimal,
    /// A hexadecimal number using `{:x}`, `{:X}`, `{:#x}`, or `{:#X}`. Prefix (if allowed) is `0x` or `0X`.
    Hexadecimal(NumberPrefixPolicy),
    /// A custom base number using `{:r2}`..=`{:r36}` or `{:R2}`..=`{:R36}`. No prefix allowed.
    ///
    /// The value will be in the range `2..=36`, but without `2`, `8`, `10`, `16`. Use [`NumberFormatOption::to_number`] to
    /// get the full range.
    Custom(u32),
}

impl NumberFormatOption {
    /// Returns the base of the number format. The number will be in the range `2..=36`.
    ///
    /// Note that the type of the number is `u32` despite being rather small, since the
    /// [`std::<number>::from_str_radix`](u8::from_str_radix) functions take a `u32` as the base.
    pub fn to_number(self) -> u32 {
        match self {
            Self::Binary(_) => 2,
            Self::Octal(_) => 8,
            Self::Decimal => 10,
            Self::Hexadecimal(_) => 16,
            Self::Custom(base) => base,
        }
    }

    /// Returns the prefix policy of the number format.
    ///
    /// Will return [`NumberPrefixPolicy::Forbidden`] for [`Decimal`](NumberFormatOption::Decimal) and
    /// [`Custom`](NumberFormatOption::Custom).
    pub fn prefix_policy(self) -> NumberPrefixPolicy {
        match self {
            Self::Binary(policy) | Self::Octal(policy) | Self::Hexadecimal(policy) => policy,
            Self::Decimal | Self::Custom(_) => NumberPrefixPolicy::Forbidden,
        }
    }
}

/// The possible policies for the prefix of [`NumberFormatOption`].
///
/// The following table shows which prefixes (hexadecimal in this case) are allowed for each policy:
///
/// | Policy    | `123abc` | `0x123abc` | `0X123abc` |
/// |-----------|:--------:|:----------:|:----------:|
/// | Forbidden | x        |            |            |
/// | Optional  | x        | x          | x          |
/// | Required  |          | x          | x          |
///
/// There is currently no option to distinguish between lowercase and uppercase prefixes. Base parsing is currently
/// fully case-insensitive, for both the prefix and any letters in the number itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberPrefixPolicy {
    /// No prefix is allowed, just the number
    Forbidden,
    /// The prefix is optional
    Optional,
    /// The prefix is required and must be present
    Required,
}
