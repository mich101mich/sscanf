/// A Trait used by `scanf` to obtain the Regex of a Type
///
/// Has one associated Constant: `REGEX`, which should be set to a regular Expression.
/// Implement this trait for a Type that you want to be parsed using scanf.
///
/// The Regular Expression should match the string representation as exactly as possible.
/// Any incorrect matches might be caught in the from_str parsing, but that might cause this
/// regex to take characters that could have been matched by other placeholders, leading to
/// unexpected parsing failures. Also: Since `scanf` only returns an `Option` it will just say
/// `None` whether the regex matching failed or the parsing failed, so you should avoid parsing
/// failures by writing a proper regex as much as possible.
///
/// ## Example
/// Let's say we want to add a Fraction parser
/// ```
/// # #[derive(Debug, PartialEq)]
/// struct Fraction(isize, usize);
/// ```
/// Which can be obtained from any string of the kind `±X/Y` or just `X`
/// ```
/// # #[derive(Debug, PartialEq)]
/// # struct Fraction(isize, usize);
/// impl sscanf::RegexRepresentation for Fraction {
///     /// matches an optional '-' or '+' followed by a number.
///     /// possibly with a '/' and another Number
///     const REGEX: &'static str = r"[-+]?\d+(/\d+)?";
/// }
/// impl std::str::FromStr for Fraction {
///     type Err = std::num::ParseIntError;
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         let mut iter = s.split('/');
///         let num = iter.next().unwrap().parse::<isize>()?;
///         let mut denom = 1;
///         if let Some(d) = iter.next() {
///             denom = d.parse::<usize>()?;
///         }
///         Ok(Fraction(num, denom))
///     }
/// }
/// ```
/// Now we can use this `Fraction` struct in `scanf`:
/// ```
/// # #[derive(Debug, PartialEq)]
/// # struct Fraction(isize, usize);
/// # impl sscanf::RegexRepresentation for Fraction {
/// #     const REGEX: &'static str = r"[-+]?\d+(/\d+)?";
/// # }
/// # impl std::str::FromStr for Fraction {
/// #     type Err = std::num::ParseIntError;
/// #     fn from_str(s: &str) -> Result<Self, Self::Err> {
/// #         let mut iter = s.split('/');
/// #         let num = iter.next().unwrap().parse::<isize>()?;
/// #         let mut denom = 1;
/// #         if let Some(d) = iter.next() {
/// #             denom = d.parse::<usize>()?;
/// #         }
/// #         Ok(Fraction(num, denom))
/// #     }
/// # }
/// use sscanf::scanf;
///
/// let output = scanf!("2/5", "{}", Fraction);
/// assert_eq!(output, Some(Fraction(2, 5)));
///
/// let output = scanf!("-25/3", "{}", Fraction);
/// assert_eq!(output, Some(Fraction(-25, 3)));
///
/// let output = scanf!("8", "{}", Fraction);
/// assert_eq!(output, Some(Fraction(8, 1)));
///
/// let output = scanf!("6e/3", "{}", Fraction);
/// assert_eq!(output, None);
///
/// let output = scanf!("6/-3", "{}", Fraction);
/// assert_eq!(output, None); // only first number can be negative
///
/// let output = scanf!("6/3", "{}", Fraction);
/// assert_eq!(output, Some(Fraction(6, 3)));
/// ```
pub trait RegexRepresentation {
    /// A regular Expression that exactly matches any String representation of the implementing Type
    const REGEX: &'static str;
}

macro_rules! doc_concat {
    ($target: item, $($doc: expr),+) => {
        $(
            #[doc = $doc]
        )+
        $target
    };
}

doc_concat! {const _A: u8 = 5;, "hi", "you", stringify!(4), "bob"}

macro_rules! impl_num {
    (u64; $(($ty: ty, $n: literal)),+) => {
        $(impl RegexRepresentation for $ty {
            doc_concat!{
                const REGEX: &'static str = concat!(r"\+?\d{1,", $n, "}");,
                "Matches any positive number with up to", stringify!($n), "digits\n",
                "```",
                "# use sscanf::RegexRepresentation;",
                concat!("assert_eq!(", stringify!($ty), "::REGEX, r\"\\+?\\d{1,", $n, "}\");"),
                "```"
            }
        })+
    };
    (i64; $(($ty: ty, $n: literal)),+) => {
        $(impl RegexRepresentation for $ty {
            doc_concat!{
                const REGEX: &'static str = concat!(r"[-+]?\d{1,", $n, "}");,
                "Matches any number with up to", stringify!($n), "digits\n",
                "```",
                "# use sscanf::RegexRepresentation;",
                concat!("assert_eq!(", stringify!($ty), r#"::REGEX, r"[-+]?\d{1,"#, $n, r#"}");"#),
                "```"
            }
        })+
    };
    (NonZeroU64; $(($ty: ty, $n: literal, $doc: literal)),+) => {
        $(impl RegexRepresentation for $ty {
            doc_concat!{
                const REGEX: &'static str = concat!(r"\+?[1-9]\d{0,", $n, "}");,
                "Matches any positive non-zero number with up to", stringify!($doc), "digits\n",
                "```",
                "# use sscanf::RegexRepresentation; use std::num::*;",
                concat!("assert_eq!(", stringify!($ty), r#"::REGEX, r"\+?[1-9]\d{0,"#, $n, r#"}");"#),
                "```"
            }
        })+
    };
    (NonZeroI64; $(($ty: ty, $n: literal, $doc: literal)),+) => {
        $(impl RegexRepresentation for $ty {
            doc_concat!{
                const REGEX: &'static str = concat!(r"[-+]?[1-9]\d{0,", $n, "}");,
                "Matches any non-zero number with up to", stringify!($doc), "digits\n",
                "```",
                "# use sscanf::RegexRepresentation; use std::num::*;",
                concat!("assert_eq!(", stringify!($ty), r#"::REGEX, r"[-+]?[1-9]\d{0,"#, $n, r#"}");"#),
                "```"
            }
        })+
    };
    (f64; $($ty: ty),+) => {
        $(impl RegexRepresentation for $ty {
            doc_concat!{
                const REGEX: &'static str = r"[-+]?\d+\.?\d*";,
                "Matches any floating point number",
                "```",
                "# use sscanf::RegexRepresentation;",
                concat!("assert_eq!(", stringify!($ty), r#"::REGEX, r"[-+]?\d+\.?\d*");"#),
                "```"
            }
        })+
    };
}

use std::num::*;

impl_num!(u64;
    (u8, 3),
    (u16, 5),
    (u32, 10),
    (u64, 20),
    (u128, 39),
    (usize, 20)
);
impl_num!(NonZeroU64;
    (NonZeroU8, 2, 3),
    (NonZeroU16, 4, 5),
    (NonZeroU32, 9, 10),
    (NonZeroU64, 19, 20),
    (NonZeroU128, 38, 39),
    (NonZeroUsize, 19, 20)
);
impl_num!(i64;
    (i8, 3),
    (i16, 5),
    (i32, 10),
    (i64, 20),
    (i128, 39),
    (isize, 20)
);
impl_num!(NonZeroI64;
    (NonZeroI8, 2, 3),
    (NonZeroI16, 4, 5),
    (NonZeroI32, 9, 10),
    (NonZeroI64, 19, 20),
    (NonZeroI128, 38, 39),
    (NonZeroIsize, 19, 20)
);
impl_num!(f64; f32, f64);

impl RegexRepresentation for String {
    /// Matches any sequence of Characters.
    /// ```
    /// # use sscanf::RegexRepresentation;
    /// assert_eq!(String::REGEX, r".+")
    /// ```
    const REGEX: &'static str = r".+";
}
impl RegexRepresentation for char {
    /// Matches a single Character.
    /// ```
    /// # use sscanf::RegexRepresentation;
    /// assert_eq!(char::REGEX, r".")
    /// ```
    const REGEX: &'static str = r".";
}
impl RegexRepresentation for bool {
    /// Matches `true` or `false`.
    /// ```
    /// # use sscanf::RegexRepresentation;
    /// assert_eq!(bool::REGEX, r"true|false")
    /// ```
    const REGEX: &'static str = r"true|false";
}

impl RegexRepresentation for std::path::PathBuf {
    /// Matches any sequence of Characters.
    ///
    /// Paths in `std` don't actually have any restrictions on what they can contain, so anything
    /// is valid.
    /// ```
    /// # use sscanf::RegexRepresentation; use std::path::PathBuf;
    /// assert_eq!(PathBuf::REGEX, r".+")
    /// ```
    const REGEX: &'static str = r".+";
}
