use std::error;
use std::str::FromStr;

#[doc(hidden)]
pub const EXPECT_NEXT_HINT: &str = r#"sscanf: Invalid number of capture groups in regex.
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid forming a capture group like this:
    "  (  )  "  =>  "  (?:  )  ""#;

#[doc(hidden)]
pub const EXPECT_CAPTURE_HINT: &str = r#"sscanf: Non-optional capture group marked as optional.
This is either a problem with a custom regex or RegexRepresentation implementation or an internal error."#;

/// The Error returned by [`scanf`](crate::scanf).
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
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::MatchFailed => None,
            Error::ParsingFailed(err) => Some(err.as_ref()),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MatchFailed => write!(f, "scanf: The input did not match the format string"),
            Error::ParsingFailed(e) => write!(f, "scanf: Parsing failed: {}", e),
        }
    }
}

/// Error type for blanket implementations of [`FromScanf`](crate::FromScanf) on [`FromStr`] types.
pub struct FromStrFailedError<T: FromStr>
where
    <T as FromStr>::Err: error::Error + 'static,
{
    /// Type name of the type that failed to parse
    type_name: &'static str,
    /// Error that was returned by the [`FromStr`] impl
    error: <T as FromStr>::Err,
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

impl<T: FromStr> std::fmt::Display for FromStrFailedError<T>
where
    <T as FromStr>::Err: error::Error + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type {} failed to parse from a string: {}",
            self.type_name, self.error
        )
    }
}
impl<T: FromStr> std::fmt::Debug for FromStrFailedError<T>
where
    <T as FromStr>::Err: error::Error + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Error type for derived [`FromScanf`](crate::FromScanf) implementations
#[derive(Debug)]
pub struct FromScanfFailedError {
    /// Type name of the type that failed to parse
    pub type_name: &'static str,
    /// Error that was returned by the underlying impl
    pub error: Box<dyn error::Error>,
}

impl std::fmt::Display for FromScanfFailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type {} failed to parse: {}", self.type_name, self.error)
    }
}

impl error::Error for FromScanfFailedError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(self.error.as_ref())
    }
}
