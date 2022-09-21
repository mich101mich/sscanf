/// A Trait used by `scanf` to obtain the Regex of a Type
///
/// Has one associated Constant: `REGEX`, which should be set to a regular Expression.
/// Implement this trait for a Type that you want to be parsed using scanf.
///
/// The Regular Expression should match the string representation as exactly as possible.
/// Any incorrect matches might be caught in the from_str parsing, but that might cause this
/// regex to take characters that could have been matched by other placeholders, leading to
/// unexpected parsing failures.
///
/// TODO: Talk abound concatcp!() and formatcp!()
///
/// **Note:** The parser uses indexing to access capture groups. To avoid messing with the
/// indexing, the regex should not contain any capture groups by using the `(?:)` syntax
/// on any round brackets:
///
/// Any `(<content>)` should be replaced with `(?:<content>)`
///
/// ## Example
/// Let's say we want to add a Fraction parser
/// ```
/// use sscanf::FromScanf;
/// #[derive(FromScanf)]
/// # #[derive(Debug, PartialEq)]
/// #[scanf(format = "{}/{}")] // placeholders are automatically indexed in order
/// struct Fraction(isize, usize);
/// ```
/// Which can be obtained from any string of the kind `±X/Y`
///
/// Now we can use this `Fraction` struct in `scanf`:
/// ```
/// # use sscanf::FromScanf;
/// # #[derive(Debug, PartialEq, FromScanf)]
/// #[scanf(format = "{}/{}")]
/// # struct Fraction(isize, usize);
/// use sscanf::scanf;
///
/// let output = scanf!("2/5", "{Fraction}");
/// assert_eq!(output.unwrap(), Fraction(2, 5));
///
/// let output = scanf!("-25/3", "{Fraction}");
/// assert_eq!(output.unwrap(), Fraction(-25, 3));
///
/// let output = scanf!("6e/3", "{Fraction}");
/// assert!(output.is_err());
///
/// let output = scanf!("6/-3", "{Fraction}");
/// assert!(output.is_err()); // only first number can be negative
///
/// let output = scanf!("6/3", "{Fraction}");
/// assert_eq!(output.unwrap(), Fraction(6, 3));
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
    /// used when calling scanf!() because of proc-macro limitations. The type returned by scanf!()
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
