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
        impl PartialEq<$name> for $target {
            fn eq(&self, rhs: &$name) -> bool {
                self.eq(&rhs.0)
            }
        }
    };
}

/// An obsolete type, currently identical to f32
#[derive(Clone, Copy, Debug, PartialEq)]
#[deprecated(since = "0.4.0", note = "use f32 instead")]
pub struct FullF32(pub f32);

impl std::str::FromStr for FullF32 {
    type Err = <f32 as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}
impl RegexRepresentation for FullF32 {
    /// Same as f32
    const REGEX: &'static str = <f32 as RegexRepresentation>::REGEX;
}
impl_wrapper_ops!(FullF32, f32);

/// An obsolete type, currently identical to f64
#[derive(Clone, Copy, Debug, PartialEq)]
#[deprecated(since = "0.4.0", note = "use f64 instead")]
pub struct FullF64(pub f64);

impl std::str::FromStr for FullF64 {
    type Err = <f64 as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}
impl RegexRepresentation for FullF64 {
    /// Same as f64
    const REGEX: &'static str = <f64 as RegexRepresentation>::REGEX;
}
impl_wrapper_ops!(FullF64, f64);

/// Matches a Hexadecimal Number with optional `0x` prefix. Deprecated in favor of format options
///
/// ```
/// # use sscanf::*;
/// let input = "deadbeef + 0x12345abc";
/// let output = sscanf!(input, "{} + {}", HexNumber, HexNumber).unwrap();
/// assert_eq!(output.0, 0xdeadbeef);
/// assert_eq!(output.1, 0x12345abc);
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
