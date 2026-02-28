//! In-memory representation of format options.

use std::borrow::Cow;

/// Possible values for the format option in a [`FromScanf`][crate::FromScanf] implementation.
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
    /// Normally a `&'static str` borrowing from the format string. Defined as a [`Cow`] for cases where custom
    /// implementations need to pass a custom format string to a subtype.
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

/// Builder for [`FormatOptions`], since it is `#[non_exhaustive]` and cannot be constructed directly.
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
    ///
    /// # Panics
    ///
    /// Panics if the radix is not in the range `2..=36`.
    #[track_caller]
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
    ///
    /// # Panics
    ///
    /// Panics if the configured options are inconsistent, e.g. if a prefix is required/optional but the number format
    /// (decimal or custom radix) does not support prefixes.
    #[track_caller]
    pub fn build(self) -> FormatOptions {
        use NumberFormatOption::*;
        use NumberPrefixPolicy::*;
        FormatOptions {
            number: match (self.radix, self.prefix) {
                (2, policy) => Binary(policy),
                (8, policy) => Octal(policy),
                (10, Forbidden) => Decimal,
                (10, _) => panic!("Decimal format (the default) does not allow a prefix"),
                (16, policy) => Hexadecimal(policy),
                (radix, Forbidden) => Other(CustomRadix::new(radix).unwrap()), // unwrap: The checks in custom_radix and this match arm ensure that this cannot fail
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
    Other(CustomRadix),
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
            Self::Other(base) => base.0,
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
    /// ```
    /// # use sscanf::advanced::{NumberFormatOption, NumberPrefixPolicy};
    /// let fmt = NumberFormatOption::Hexadecimal(NumberPrefixPolicy::Optional);
    /// assert_eq!(fmt.prefix(), Some("0x"));
    ///
    /// let fmt = NumberFormatOption::Decimal;
    /// assert_eq!(fmt.prefix(), None);
    ///
    /// let fmt = NumberFormatOption::Hexadecimal(NumberPrefixPolicy::Forbidden);
    /// assert_eq!(fmt.prefix(), None); // no prefix allowed
    /// ```
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

/// A custom radix for a number format.
///
/// This type guarantees that the radix is in the range `2..=36` and not `2`, `8`, `10`, or `16`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomRadix(u32);

impl CustomRadix {
    /// Creates a new `CustomRadix` if the value is valid.
    ///
    /// # Errors
    ///
    /// Returns an error if the radix is not in `2..=36`, or if it is one of the standard bases
    /// `2`, `8`, `10`, or `16` (which should use the specific variants of [`NumberFormatOption`] instead).
    pub const fn new(radix: u32) -> Result<Self, &'static str> {
        if radix < 2 || radix > 36 {
            return Err("Radix must be in the range 2..=36");
        }
        if matches!(radix, 2 | 8 | 10 | 16) {
            return Err("Radix 2, 8, 10, and 16 are covered by other variants");
        }
        Ok(Self(radix))
    }

    /// Returns the value of the custom radix.
    pub fn value(self) -> u32 {
        self.0
    }
}

/// The possible policies for the prefix of [`NumberFormatOption`].
///
/// The following table shows which prefixes (hexadecimal in this case) are allowed for each policy:
///
/// | Policy    | `123abc` | `0x123abc` | `0X123abc` |
/// |-----------|:--------:|:----------:|:----------:|
/// | Forbidden | yes      |            |            |
/// | Optional  | yes      | yes        | yes        |
/// | Required  |          | yes        | yes        |
///
/// There is currently no option to distinguish between lowercase and uppercase prefixes. Base parsing is currently
/// fully case-insensitive, for both the prefix and any letters in the number itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberPrefixPolicy {
    /// No prefix is allowed; just the number.
    Forbidden,
    /// The prefix is optional.
    Optional,
    /// The prefix is required and must be present.
    Required,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_radix_new() {
        assert!(CustomRadix::new(1).is_err());
        assert!(CustomRadix::new(3).is_ok());
        assert!(CustomRadix::new(36).is_ok());
        assert!(CustomRadix::new(37).is_err());

        assert!(CustomRadix::new(2).is_err());
        assert!(CustomRadix::new(8).is_err());
        assert!(CustomRadix::new(10).is_err());
        assert!(CustomRadix::new(16).is_err());

        let valid_ranges = [3..=7, 9..=9, 11..=15, 17..=36];
        for range in valid_ranges {
            for radix in range {
                assert!(
                    CustomRadix::new(radix).is_ok(),
                    "CustomRadix::new({radix}) should be valid"
                );
            }
        }
    }

    #[test]
    fn test_to_number() {
        let options = NumberFormatOption::Binary(NumberPrefixPolicy::Forbidden);
        assert_eq!(options.to_number(), 2);

        let options = NumberFormatOption::Octal(NumberPrefixPolicy::Forbidden);
        assert_eq!(options.to_number(), 8);

        let options = NumberFormatOption::Decimal;
        assert_eq!(options.to_number(), 10);

        let options = NumberFormatOption::Hexadecimal(NumberPrefixPolicy::Forbidden);
        assert_eq!(options.to_number(), 16);

        let radix = CustomRadix::new(3).unwrap();
        let options = NumberFormatOption::Other(radix);
        assert_eq!(options.to_number(), 3);
    }

    #[test]
    fn test_builder_mapping() {
        // Builder should map 2, 8, 10, 16 to appropriate variants
        let opts = FormatOptions::builder().custom_radix(2).build();
        assert!(matches!(opts.number, NumberFormatOption::Binary(_)));

        let opts = FormatOptions::builder().custom_radix(8).build();
        assert!(matches!(opts.number, NumberFormatOption::Octal(_)));

        let opts = FormatOptions::builder().custom_radix(10).build();
        assert!(matches!(opts.number, NumberFormatOption::Decimal));

        let opts = FormatOptions::builder().custom_radix(16).build();
        assert!(matches!(opts.number, NumberFormatOption::Hexadecimal(_)));
    }

    #[test]
    fn test_builder_custom() {
        // Custom radix 3 is valid and handled as Other
        let opts = FormatOptions::builder().custom_radix(3).build();
        assert_eq!(opts.number, NumberFormatOption::Other(CustomRadix(3)));
    }

    #[test]
    fn test_prefix_policies() {
        // Test that prefix policies are set correctly
        let opts = FormatOptions::builder()
            .binary()
            .with_optional_prefix()
            .build();
        assert_eq!(opts.number.prefix_policy(), NumberPrefixPolicy::Optional);
        assert_eq!(opts.number.prefix(), Some("0b"));

        let opts = FormatOptions::builder().octal().with_prefix().build();
        assert_eq!(opts.number.prefix_policy(), NumberPrefixPolicy::Required);
        assert_eq!(opts.number.prefix(), Some("0o"));

        let opts = FormatOptions::builder().hex().build();
        assert_eq!(opts.number.prefix_policy(), NumberPrefixPolicy::Forbidden);
        assert_eq!(opts.number.prefix(), None);
    }

    #[test]
    #[should_panic(expected = "Custom radix 3 does not allow a prefix")]
    fn test_builder_custom_prefix_panic() {
        FormatOptions::builder()
            .custom_radix(3)
            .with_optional_prefix()
            .build();
    }

    #[test]
    #[should_panic(expected = "Decimal format (the default) does not allow a prefix")]
    fn test_builder_decimal_prefix_panic() {
        FormatOptions::builder()
            .decimal()
            .with_optional_prefix()
            .build();
    }
}
