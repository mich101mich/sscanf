use crate::RegexRepresentation;
use std::ops::*;
use std::str::FromStr;

macro_rules! impl_wrapper_ops {
    ($name: ty, $target: ty) => {
        impl_wrapper_ops!($name, $target, <>);
    };
    ($name: ty, $target: ty, $($generics: tt)+) => {
        impl $($generics)+ Deref for $name {
            type Target = $target;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl $($generics)+ DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
        impl $($generics)+ PartialEq<$target> for $name where $target: PartialEq {
            fn eq(&self, rhs: &$target) -> bool {
                self.0.eq(rhs)
            }
        }
        impl $($generics)+ PartialEq<$name> for $target where $target: PartialEq {
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
/// let output = scanf!(input, "{}{}{}", String, f32, String);
/// // There are no Numbers in input, so expect None
/// assert!(output.is_none());
///
/// let output = scanf!(input, "{}{}{}", String, FullF32, String);
/// // The 'nan' part in "Banana" is parsed as f32::NaN
/// assert!(output.is_some());
/// assert!(output.unwrap().1.is_nan());
/// ```
///
/// See [FromStr on f32](https://doc.rust-lang.org/std/primitive.f32.html#impl-FromStr) for the
/// full syntax
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FullF32(pub f32);

impl FromStr for FullF32 {
    type Err = <f32 as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = if s.to_lowercase().ends_with("nan") {
            if s.len() == 4 {
                -f32::NAN
            } else {
                f32::NAN
            }
        } else {
            s.to_lowercase().parse()?
        };
        Ok(FullF32(f))
    }
}
impl RegexRepresentation for FullF32 {
    /// Matches any floating point number, including `nan`, `inf`, `2.0e5`, ...
    ///
    /// See [FromStr on f32](https://doc.rust-lang.org/std/primitive.f32.html#impl-FromStr) for details
    fn regex() -> &'static str {
        r"[-+]?([nN]a[nN]|[iI]nf|(\d+|\d+\.\d*|\d*\.\d+)([eE][-+]?\d+)?)"
    }
}
impl_wrapper_ops!(FullF32, f32);

/// A Wrapper around f64 whose RegexRepresentation also includes special floating point values
/// like `nan`, `inf`, `2.0e5`, ...
///
/// See [`FullF32`] for Details
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FullF64(pub f64);

impl FromStr for FullF64 {
    type Err = <f64 as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = if s.to_lowercase().ends_with("nan") {
            if s.len() == 4 {
                -f64::NAN
            } else {
                f64::NAN
            }
        } else {
            s.to_lowercase().parse()?
        };
        Ok(FullF64(f))
    }
}
impl RegexRepresentation for FullF64 {
    /// Matches any floating point number, including `nan`, `inf`, `2.0e5`, ...
    fn regex() -> &'static str {
        FullF32::regex()
    }
}
impl_wrapper_ops!(FullF64, f64);

/// Matches a Hexadecimal Number with optional `0x` prefix
///
/// ```
/// # use sscanf::*;
/// let input = "deadbeef + 0x123456789abcdef";
/// let output = scanf!(input, "{} + {}", HexNumber, HexNumber).unwrap();
/// assert_eq!(output.0, 0xdeadbeef);
/// assert_eq!(output.1, 0x123456789abcdef);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HexNumber(pub usize);

impl FromStr for HexNumber {
    type Err = <usize as FromStr>::Err;
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
    fn regex() -> &'static str {
        r"0[xX][0-9a-fA-F]+|[0-9a-fA-F]+"
    }
}
impl_wrapper_ops!(HexNumber, usize);

/// A Wrapper around Vec
#[derive(Clone, Debug)]
pub struct VecWrapper<T: 'static + RegexRepresentation + FromStr>(pub Vec<T>);

use std::{any::TypeId, collections::HashMap};

impl<T: 'static + RegexRepresentation + FromStr> RegexRepresentation for VecWrapper<T> {
    fn regex() -> &'static str {
        static mut STR: Option<HashMap<TypeId, String>> = None;
        unsafe { STR.get_or_insert_with(HashMap::new) }
            .entry(TypeId::of::<T>())
            .or_insert_with(|| String::from(r"\[((") + T::regex() + r"), ?)*(" + T::regex() + r")?\]")
            .as_str()
    }
}

use regex::Regex;
impl<T: 'static + RegexRepresentation + FromStr> FromStr for VecWrapper<T> {
    type Err = <T as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static mut REGEX: Option<HashMap<TypeId, Regex>> = None;
        let regex = unsafe { REGEX.get_or_insert_with(HashMap::new) }
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Regex::new(T::regex()).unwrap());

        let mut ret = vec![];
        let s = s
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .unwrap();

        for s in regex.find_iter(s) {
            ret.push(T::from_str(s.as_str())?)
        }
        Ok(VecWrapper(ret))
    }
}

impl_wrapper_ops!(VecWrapper<T>, Vec<T>, <T: RegexRepresentation + FromStr>);
