use std::borrow::Cow;

/// The possible values for the format option in a [`FromScanf`][super::FromScanf] implementation.
///
/// Note that there is also the custom regex override (e.g. `{:/[a-d]+/}`), but that is handled externally by the
/// macros and not passed on to the types.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct FormatOptions {
    /// A formatter for a number (e.g. `{:x}`). If not present, defaults to base 10.
    pub number: NumberFormatOption,
    /// A fully custom format string (e.g. `{:[%Y-%m-%d]}` for a chrono date).
    ///
    /// Normally just a `&'static str` borrowing from the format string. Defined as a [`Cow`] just in case some custom
    /// implementations needs to pass a custom-custom format string to a subtype.
    pub custom: Option<Cow<'static, str>>,
}

/// The possible number formats for a number formatter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumberFormatOption {
    /// A binary number using `{:b}`, `{:#b}`, or `{:r2}`. Prefix (if allowed) is `0b` or `0B`.
    Binary(NumberPrefixPolicy),
    /// An octal number using `{:o}`, `{:#o}`, or `{:r8}`. Prefix (if allowed) is `0o` or `0O`.
    Octal(NumberPrefixPolicy),
    /// A decimal number. No prefix allowed. The default number format.
    #[default]
    Decimal,
    /// A hexadecimal number using `{:x}`, `{:#x}`, or `{:r16}`. Prefix (if allowed) is `0x` or `0X`.
    Hexadecimal(NumberPrefixPolicy),
    /// A custom base number using `{:r2}`..=`{:r36}`. No prefix allowed.
    ///
    /// The value will be in the range `2..=36`, but without `2`, `8`, `10`, `16`. Formats like `{:r2}` will
    /// be mapped to `Binary(NumberPrefixPolicy::Forbidden)` to simplify the implementation for types that only care
    /// about the usual bases. Types that deal with arbitrary bases can call [`NumberFormatOption::to_number`] to
    /// get the base as a number.
    Other(u32),
}

impl NumberFormatOption {
    /// Returns the base of the number format. The number will be in the range `2..=36`.
    ///
    /// Note that the type of the number is `u32` despite fitting in a smaller type, since the
    /// [`std::<number>::from_str_radix`](u8::from_str_radix) functions take a `u32` as the base.
    pub fn to_number(self) -> u32 {
        match self {
            Self::Binary(_) => 2,
            Self::Octal(_) => 8,
            Self::Decimal => 10,
            Self::Hexadecimal(_) => 16,
            Self::Other(base) => base,
        }
    }

    /// Returns the prefix policy of the number format.
    ///
    /// Will return [`NumberPrefixPolicy::Forbidden`] for [`Decimal`](NumberFormatOption::Decimal) and
    /// [`Other`](NumberFormatOption::Other).
    pub fn prefix_policy(self) -> NumberPrefixPolicy {
        match self {
            Self::Binary(policy) | Self::Octal(policy) | Self::Hexadecimal(policy) => policy,
            Self::Decimal | Self::Other(_) => NumberPrefixPolicy::Forbidden,
        }
    }

    /// Returns the prefix for the number format, if any.
    ///
    /// E.g. for [`Binary`](NumberFormatOption::Binary) it will return `Some("0b")`.
    pub fn prefix(self) -> Option<&'static str> {
        use NumberPrefixPolicy::*;
        match self {
            Self::Binary(Optional | Required) => Some("0b"),
            Self::Octal(Optional | Required) => Some("0o"),
            Self::Hexadecimal(Optional | Required) => Some("0x"),
            _ => None,
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
