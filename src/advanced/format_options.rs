//! In-memory representations of the format options

use std::borrow::Cow;

/// The possible values for the format option in a [`FromScanf`][crate::FromScanf] implementation.
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

impl FormatOptions {
    /// Creates a builder for the [`FormatOptions`] struct.
    ///
    /// Default settings:
    /// - Radix: 10 (decimal)
    /// - Prefix: no prefix allowed
    /// - Custom format: None
    pub fn builder() -> Builder {
        Builder {
            radix: 10,
            prefix: NumberPrefixPolicy::Forbidden,
            custom: None,
        }
    }
}

/// A builder for the [`FormatOptions`] struct, since it is marked as `#[non_exhaustive]` and thus cannot be
/// constructed directly.
#[derive(Clone, Debug)]
pub struct Builder {
    radix: u32,
    prefix: NumberPrefixPolicy,
    custom: Option<Cow<'static, str>>,
}

impl Builder {
    /// Sets the radix for the number format to binary.
    ///
    /// Note that this does not change the prefix policy.
    pub fn binary(mut self) -> Self {
        self.radix = 2;
        self
    }
    /// Sets the radix for the number format to octal.
    ///
    /// Note that this does not change the prefix policy.
    pub fn octal(mut self) -> Self {
        self.radix = 8;
        self
    }
    /// Sets the radix for the number format to decimal (the default).
    ///
    /// Note that this does not change the prefix policy.
    pub fn decimal(mut self) -> Self {
        self.radix = 10;
        self
    }
    /// Sets the radix for the number format to hexadecimal.
    ///
    /// Note that this does not change the prefix policy.
    pub fn hex(mut self) -> Self {
        self.radix = 16;
        self
    }
    /// Sets the radix for the number format to a custom base.
    ///
    /// The base must be in the range `2..=36`.
    /// Note that this does not change the prefix policy.
    pub fn custom_radix(mut self, radix: u32) -> Self {
        if !(2..=36).contains(&radix) {
            panic!("Radix must be in the range 2..=36, got {radix}");
        }
        self.radix = radix;
        self
    }

    /// Sets the prefix policy to an optional prefix.
    pub fn with_optional_prefix(mut self) -> Self {
        self.prefix = NumberPrefixPolicy::Optional;
        self
    }
    /// Sets the prefix policy to a required prefix.
    pub fn with_prefix(mut self) -> Self {
        self.prefix = NumberPrefixPolicy::Required;
        self
    }

    /// Sets the custom format string.
    pub fn with_custom_string(mut self, custom: impl Into<Cow<'static, str>>) -> Self {
        self.custom = Some(custom.into());
        self
    }

    /// Builds the [`FormatOptions`] struct.
    pub fn build(self) -> FormatOptions {
        use NumberFormatOption::*;
        use NumberPrefixPolicy::*;
        FormatOptions {
            number: match (self.radix, self.prefix) {
                (2, policy) => Binary(policy),
                (8, policy) => Octal(policy),
                (10, Forbidden) => Decimal,
                (10, _) => panic!("Decimal format does not allow a prefix"),
                (16, policy) => Hexadecimal(policy),
                (radix, Forbidden) => Other(radix),
                (radix, _) => panic!("Custom radix {radix} does not allow a prefix"),
            },
            custom: self.custom,
        }
    }
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
