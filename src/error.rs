use std::error::Error;

/// The Error returned by scanf if the input does not match the format string.
#[derive(Debug, Clone, Copy)]
pub struct ScanfMatchFailed;

impl Error for ScanfMatchFailed {}

impl std::fmt::Display for ScanfMatchFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "scanf: The input did not match the format string")
    }
}

use std::str::FromStr;

/// Error type for [`FromScanf`](crate::FromScanf) implementation for
/// [`FromStr`](std::str::FromStr) types.
pub struct FromStrFailedError<T>
where
    T: FromStr,
    <T as FromStr>::Err: Error,
{
    /// Type name of the type that failed to parse
    pub type_name: &'static str,
    /// Error that was returned by the `FromStr` impl
    pub error: <T as FromStr>::Err,
}

impl<T: FromStr> std::fmt::Display for FromStrFailedError<T>
where
    <T as FromStr>::Err: Error,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type {} failed to parse from string: {}",
            self.type_name, self.error
        )
    }
}
impl<T: FromStr> std::fmt::Debug for FromStrFailedError<T>
where
    <T as FromStr>::Err: Error,
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
impl<T: FromStr> Error for FromStrFailedError<T> where <T as FromStr>::Err: Error {}

/// Error type for `FromScanf` impls that wrap around other `FromScanf` impls
#[derive(Debug)]
pub struct FromScanfFailedError {
    /// Type name of the type that failed to parse
    pub type_name: &'static str,
    /// Error that was returned by the underlying impl
    pub error: Box<dyn Error>,
}

impl std::fmt::Display for FromScanfFailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type {} failed to parse: {}", self.type_name, self.error)
    }
}

impl Error for FromScanfFailedError {}
