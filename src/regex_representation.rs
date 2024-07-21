use const_format::formatcp;

/// A Trait used by `sscanf` to obtain the Regex of a Type
///
/// Has one associated Constant: `REGEX`, which should be set to a regular Expression.
/// Implement this trait for a Type that you want to be parsed using sscanf.
///
/// The Regular Expression should match the string representation as exactly as possible.
/// Any incorrect matches might be caught in the from_str parsing, but that might cause this
/// regex to take characters that could have been matched by other placeholders, leading to
/// unexpected parsing failures.
///
/// ## Implementing the Trait
///
/// A manual implementation of this trait is only necessary if you
/// - want to use a custom Type that is not supported by default **AND**
/// - cannot use [`#[derive(FromScanf)]`](derive.FromScanf.html) on your Type
///
/// Deriving [`FromScanf`](crate::FromScanf) will automatically implement this trait for your Type,
/// and should be preferred in most cases.
///
/// If you do need to implement this trait yourself, note the following:
/// - The regex cannot contain any capture groups (round brackets). If you need to use `( )` in your
///   regex, use `(?: )` instead to make it non-capturing.
/// - Using a raw string literal (`r"..."`) is recommended to avoid having to escape backslashes.
/// - The [`const_format`] crate can be used to combine multiple
///   strings into one, which is useful for complex regexes. This can also be used to combine the
///   existing regex implementation of other types. `sscanf` internally uses `const_format` as well,
///   so a version of it is re-exported under `sscanf::const_format`.
///
/// ## Example
/// Let's say we want to add a Fraction parser
/// ```
/// struct Fraction(isize, usize);
/// ```
/// Which can be obtained from any string of the kind `±X/Y` or just `X`
/// ```
/// # struct Fraction(isize, usize);
/// impl sscanf::RegexRepresentation for Fraction {
///     /// matches an optional '-' or '+' followed by a number.
///     /// possibly with a '/' and another Number
///     const REGEX: &'static str = r"[-+]?\d+(?:/\d+)?";
///     //                                     ^^ escapes the group. Has to be used on any ( ) in a regex.
///
///     // alternatively, we could use const_format to reuse existing regexes:
///     // REGEX = const_format::concatcp!(isize::REGEX, "(?:", "/", usize::REGEX, ")?");
/// }
/// impl std::str::FromStr for Fraction {
///     type Err = std::num::ParseIntError;
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         let mut iter = s.split('/');
///         let num = iter.next().unwrap().parse()?;
///         let mut denom = 1;
///         if let Some(d) = iter.next() {
///             denom = d.parse()?;
///         };
///         Ok(Fraction(num, denom))
///     }
/// }
/// ```
/// Now we can use this `Fraction` struct in `sscanf`:
/// ```
/// # #[derive(Debug, PartialEq)]
/// # struct Fraction(isize, usize);
/// # impl sscanf::RegexRepresentation for Fraction {
/// #     const REGEX: &'static str = r"[-+]?\d+(?:/\d+)?";
/// # }
/// # impl std::str::FromStr for Fraction {
/// #     type Err = std::num::ParseIntError;
/// #     fn from_str(s: &str) -> Result<Self, Self::Err> {
/// #         let mut iter = s.split('/');
/// #         let num = iter.next().unwrap().parse()?;
/// #         let denom = iter.next().map(|d| d.parse()).transpose()?.unwrap_or(1);
/// #         Ok(Fraction(num, denom))
/// #     }
/// # }
/// # use sscanf::sscanf;
///
/// let output = sscanf!("2/5", "{}", Fraction);
/// assert_eq!(output.unwrap(), Fraction(2, 5));
///
/// let output = sscanf!("-25/3", "{}", Fraction);
/// assert_eq!(output.unwrap(), Fraction(-25, 3));
///
/// let output = sscanf!("8", "{}", Fraction);
/// assert_eq!(output.unwrap(), Fraction(8, 1));
///
/// let output = sscanf!("6e/3", "{}", Fraction);
/// assert!(output.is_err());
///
/// let output = sscanf!("6/-3", "{}", Fraction);
/// assert!(output.is_err()); // only first number can be negative
///
/// let output = sscanf!("6/3", "{}", Fraction);
/// assert_eq!(output.unwrap(), Fraction(6, 3));
/// ```
pub trait RegexRepresentation {
    /// A regular Expression that exactly matches any String representation of the implementing Type
    const REGEX: &'static str;
}

// float syntax: https://doc.rust-lang.org/std/primitive.f32.html#grammar
//
// Float  ::= Sign? ( 'inf' | 'infinity' | 'nan' | Number )
const FLOAT: &str = formatcp!(r"{SIGN}?(?i:inf|infinity|nan|{NUMBER})",);
// Number ::= ( Digit+ | Digit+ '.' Digit* | Digit* '.' Digit+ ) Exp?
const NUMBER: &str = formatcp!(r"(?:{DIGIT}+|{DIGIT}+\.{DIGIT}*|{DIGIT}*\.{DIGIT}+)(?:{EXP})?",);
// Exp    ::= 'e' Sign? Digit+
const EXP: &str = formatcp!(r"e{SIGN}?{DIGIT}+");
// Sign   ::= [+-]
const SIGN: &str = r"[+-]";
// Digit  ::= [0-9]
const DIGIT: &str = r"\d";

macro_rules! doc_concat {
    ($target: item, $($doc: expr),+) => {
        $(
            #[doc = $doc]
        )+
        $target
    };
}

macro_rules! impl_num {
    ($spec: literal, $prefix: literal; $(($ty: ty, $n: literal)),+) => {
        impl_num!($spec, $prefix; $(($ty, $n, $n)),+);
    };
    ($spec: literal, $prefix: literal; $(($ty: ty, $n: literal, $doc: literal)),+) => {
        $(impl RegexRepresentation for $ty {
            doc_concat!{
                const REGEX: &'static str = concat!($prefix, $n, "}");,
                "Matches ", $spec, " number with up to", stringify!($doc), "digits\n",
                "```",
                "# use sscanf::RegexRepresentation; use std::num::*;",
                concat!("assert_eq!(", stringify!($ty), "::REGEX, r\"", $prefix, $n, "}\");"),
                "```"
            }
        })+
    };
    (f64; $($ty: ty),+) => {
        $(impl RegexRepresentation for $ty {
            doc_concat!{
                const REGEX: &'static str = FLOAT;,
                "Matches any floating point number",
                "",
                concat!("See See [FromStr on ", stringify!($ty), "](https://doc.rust-lang.org/std/primitive.", stringify!($ty), ".html#method.from_str) for details"),
                "```",
                "# use sscanf::RegexRepresentation;",
                concat!("assert_eq!(", stringify!($ty), r#"::REGEX, r"[+-]?(?i:inf|infinity|nan|(?:\d+|\d+\.\d*|\d*\.\d+)(?:e[+-]?\d+)?)");"#),
                "```"
            }
        })+
    };
}

use std::num::*;

impl_num!("any positive", r"\+?\d{1,";
    (u8, 3),
    (u16, 5),
    (u32, 10),
    (u64, 20),
    (u128, 39),
    (usize, 20)
);
impl_num!("any positive non-zero", r"\+?[1-9]\d{0,";
    (NonZeroU8, 2, 3),
    (NonZeroU16, 4, 5),
    (NonZeroU32, 9, 10),
    (NonZeroU64, 19, 20),
    (NonZeroU128, 38, 39),
    (NonZeroUsize, 19, 20)
);
impl_num!("any", r"[-+]?\d{1,";
    (i8, 3),
    (i16, 5),
    (i32, 10),
    (i64, 20),
    (i128, 39),
    (isize, 20)
);
impl_num!("any non-zero", r"[-+]?[1-9]\d{0,";
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
    ///
    /// Note that this clones part of the input string, which is usually not necessary. Use
    /// [`str`](#impl-RegexRepresentation-for-str) unless you explicitly need ownership.
    /// ```
    /// # use sscanf::RegexRepresentation;
    /// assert_eq!(String::REGEX, r".+?")
    /// ```
    const REGEX: &'static str = r".+?";
}
impl RegexRepresentation for str {
    /// Matches any sequence of Characters.
    ///
    /// Note that this is the non-borrowed form of the usual `&str`. This is the type that should be
    /// used when calling sscanf!() because of proc-macro limitations. The type returned by sscanf!()
    /// is `&str` as one would expect.
    ///
    /// This is also currently the only type that borrows part of the input string, so you need to
    /// keep lifetimes in mind when using this type. If the input string doesn't live long enough,
    /// use [`String`](#impl-RegexRepresentation-for-String) instead.
    /// ```
    /// # use sscanf::RegexRepresentation;
    /// assert_eq!(str::REGEX, r".+?")
    /// ```
    const REGEX: &'static str = r".+?";
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

#[test]
#[rustfmt::skip]
fn no_capture_groups() {
    macro_rules! check {
        ($($ty: ty),+) => {
            $(
                let regex = regex::Regex::new(<$ty>::REGEX).unwrap();
                assert_eq!(regex.captures_len(), 1, "Regex for {} >>{}<< has capture groups", stringify!($ty), <$ty>::REGEX);
                // 1 for the whole match
            )+ 
        };
    }

    check!(u8, u16, u32, u64, u128, usize);
    check!(i8, i16, i32, i64, i128, isize);
    check!(NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize);
    check!(NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize);
    check!(f32, f64);
    check!(String, str, char, bool);
    check!(std::path::PathBuf);
}
