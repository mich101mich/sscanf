#![allow(deprecated)]
use crate::RegexRepresentation;
use std::ops::*;

macro_rules! impl_wrapper_ops {
    ($name: ty, $target: ty) => {
        impl Deref for $name {
            type Target = $target;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
        impl PartialEq<$target> for $name {
            fn eq(&self, rhs: &$target) -> bool {
                self.0.eq(rhs)
            }
        }
        #[allow(rustdoc::missing_doc_code_examples)]
        impl PartialEq<$name> for $target {
            fn eq(&self, rhs: &$name) -> bool {
                self.eq(&rhs.0)
            }
        }
    };
}

/// A Wrapper around f32 whose RegexRepresentation also includes special floating point values
/// like `nan`, `inf`, `2.0e5`, ...
///
/// This is not part of the regular f32 parser because having a Number match against Text like with
/// `nan` is usually not desirable:
/// ```
/// # use sscanf::*;
/// let input = "Match a Banana against a number";
/// let output = scanf!(input, "{}{}{}", str, f32, str);
/// // There are no Numbers in input, so expect None
/// assert!(output.is_err());
///
/// let output = scanf!(input, "{}{}{}", str, FullF32, str);
/// // The 'nan' part in "Banana" is parsed as f32::NaN
/// assert!(output.is_ok());
/// assert!(output.unwrap().1.is_nan());
/// ```
///
/// See [FromStr on f32](https://doc.rust-lang.org/std/primitive.f32.html#impl-FromStr) for the
/// full syntax
#[derive(Clone, Copy, Debug, PartialEq)]
#[deprecated(
    since = "0.4.0",
    note = "use \"{f32:#}\" instead"
)]
pub struct FullF32(pub f32);

impl std::str::FromStr for FullF32 {
    type Err = <f32 as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = match s.to_lowercase().as_str() {
            "nan" => f32::NAN,
            "-nan" => -f32::NAN,
            s => s.parse()?,
        };
        Ok(FullF32(f))
    }
}
impl RegexRepresentation for FullF32 {
    /// Matches any floating point number, including `nan`, `inf`, `2.0e5`, ...
    ///
    /// See [FromStr on f32](https://doc.rust-lang.org/std/primitive.f32.html#impl-FromStr) for details
    const REGEX: &'static str = r"[-+]?(?:[nN]a[nN]|[iI]nf|(?:\d+|\d+\.\d*|\d*\.\d+)(?:[eE][-+]?\d+)?)";
}
impl_wrapper_ops!(FullF32, f32);

/// A Wrapper around f64 whose RegexRepresentation also includes special floating point values
/// like `nan`, `inf`, `2.0e5`, ...
///
/// See [`FullF32`] for Details
#[derive(Clone, Copy, Debug, PartialEq)]
#[deprecated(
    since = "0.4.0",
    note = "use \"{f64:#}\" instead"
)]
pub struct FullF64(pub f64);

impl std::str::FromStr for FullF64 {
    type Err = <f64 as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = match s.to_lowercase().as_str() {
            "nan" => f64::NAN,
            "-nan" => -f64::NAN,
            s => s.parse()?,
        };
        Ok(FullF64(f))
    }
}
impl RegexRepresentation for FullF64 {
    /// Matches any floating point number, including `nan`, `inf`, `2.0e5`, ...
    const REGEX: &'static str = FullF32::REGEX;
}
impl_wrapper_ops!(FullF64, f64);

/// Matches a Hexadecimal Number with optional `0x` prefix. Deprecated in favor of format options
///
/// ```
/// # use sscanf::*;
/// let input = "deadbeef + 0x123456789abcdef";
/// let output = scanf!(input, "{} + {}", HexNumber, HexNumber).unwrap();
/// assert_eq!(output.0, 0xdeadbeef);
/// assert_eq!(output.1, 0x123456789abcdef);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[deprecated(
    since = "0.1.3",
    note = "use \"{:x}\" with the desired number type instead"
)]
pub struct HexNumber(pub usize);

impl std::str::FromStr for HexNumber {
    type Err = <usize as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .unwrap_or(s);
        Ok(HexNumber(usize::from_str_radix(s, 16)?))
    }
}
impl RegexRepresentation for HexNumber {
    /// Matches any hexadecimal number. Can have a `0x` or `0X` prefix
    const REGEX: &'static str = r"0[xX][0-9a-fA-F]+|[0-9a-fA-F]+";
}
impl_wrapper_ops!(HexNumber, usize);
