//! Various error types used by the crate. The most important one is [`Error`], which is returned by [`sscanf`](crate::sscanf).

use std::error; // can't use `Error` directly because of naming conflict; can't alias because that would show up in docs
use std::fmt::{self, Display};
use std::str::FromStr;

#[doc(hidden)]
pub static EXPECT_NEXT_HINT: &str = r#"sscanf: Invalid number of capture groups in regex.
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid forming a capture group like this:
    "  (  )  "  =>  "  (?:  )  ""#;

#[doc(hidden)]
pub static EXPECT_CAPTURE_HINT: &str = r#"sscanf: Non-optional capture group marked as optional.
This is either a problem with a custom regex or RegexRepresentation implementation or an internal error."#;

#[doc(hidden)]
pub static WRONG_CAPTURES_HINT: &str = r#"
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid forming a capture group like this:
    "  (  )  "  =>  "  (?:  )  "
"#;

/// The Error returned by [`sscanf`](crate::sscanf).
#[derive(Debug)]
pub enum Error {
    /// The Regex generated from the format string did not match the input
    MatchFailed,
    /// One of the [`FromStr`] or [`FromScanf`](crate::FromScanf) conversions failed
    ///
    /// This variant usually indicates that a [`RegexRepresentation`](crate::RegexRepresentation)
    /// of a type allows too many values to be accepted. This cannot always be avoided without
    /// creating a ridiculously complex regex, and so this error is returned instead.
    /// In those cases, it is fine to treat this as an extension of [`MatchFailed`](Error::MatchFailed).
    ///
    /// The exact content of this error is only relevant when debugging custom implementations of
    /// [`FromStr`] or [`FromScanf`](crate::FromScanf).
    ParsingFailed(Box<dyn error::Error>),
}

impl error::Error for Error {
    /// Returns the underlying error if this is a [`ParsingFailed`](Error::ParsingFailed) error.
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::MatchFailed => None,
            Error::ParsingFailed(err) => Some(err.as_ref()),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MatchFailed => write!(f, "sscanf: The input did not match the format string"),
            Error::ParsingFailed(e) => write!(f, "sscanf: Parsing failed: {}", e),
        }
    }
}

/// Error type for blanket implementations of [`FromScanf`](crate::FromScanf) on [`FromStr`] types.
pub struct FromStrFailedError<T: FromStr>
where
    <T as FromStr>::Err: error::Error + 'static,
{
    /// Type name of the type that failed to parse (for display purposes only)
    pub type_name: &'static str,
    /// Error that was returned by the [`FromStr`] impl
    pub error: <T as FromStr>::Err,
}

impl<T: FromStr> FromStrFailedError<T>
where
    <T as FromStr>::Err: error::Error + 'static,
{
    pub(crate) fn new(error: <T as FromStr>::Err) -> Self {
        Self {
            type_name: std::any::type_name::<T>(),
            error,
        }
    }
}

impl<T: FromStr> Display for FromStrFailedError<T>
where
    <T as FromStr>::Err: error::Error + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "type {} failed to parse from a string: {}",
            self.type_name, self.error
        )
    }
}
impl<T: FromStr> fmt::Debug for FromStrFailedError<T>
where
    <T as FromStr>::Err: error::Error + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // has to be manually implemented because derive adds a `where T: Debug` bound,
        // even though T itself is not used in the Debug impl
        f.debug_struct("FromStrFailedError")
            .field("type_name", &self.type_name)
            .field("error", &self.error)
            .finish()
    }
}

impl<T: FromStr> error::Error for FromStrFailedError<T>
where
    <T as FromStr>::Err: error::Error + 'static,
{
    /// Returns the underlying [`FromStr::Err`] error
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Error type for derived [`FromScanf`](crate::FromScanf) implementations
#[derive(Debug)]
pub struct FromScanfFailedError {
    /// The name of the implementing type (for display purposes only)
    pub type_name: &'static str,
    /// Error that was returned by the underlying impl
    pub error: Box<dyn error::Error>,
}

impl Display for FromScanfFailedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "type {} failed to parse from sscanf: {}",
            self.type_name, self.error
        )
    }
}

impl error::Error for FromScanfFailedError {
    /// Returns the underlying error
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(self.error.as_ref())
    }
}

/// Error type used when using the `{:#x}` etc. format options if there was no prefix
#[derive(Debug)]
pub enum MissingPrefixError {
    /// The `0x` prefix was missing
    Hex,
    /// The `0o` prefix was missing
    Octal,
    /// The `0b` prefix was missing
    Binary,
}

impl Display for MissingPrefixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Missing prefix: {}",
            match self {
                MissingPrefixError::Hex => "0x",
                MissingPrefixError::Octal => "0o",
                MissingPrefixError::Binary => "0b",
            }
        )
    }
}

impl error::Error for MissingPrefixError {}

/// Error type used when a `[sscanf(filter_map = ...)]` closure returns `None`
#[derive(Debug)]
pub struct FilterMapNoneError {
    /// Type name of the field that the attribute is on (for display purposes only)
    pub field_name: &'static str,
}

impl Display for FilterMapNoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "The closure of `{}`s `filter_map` attribute returned None",
            self.field_name
        )
    }
}

impl error::Error for FilterMapNoneError {}
