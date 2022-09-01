use std::error::Error;
use std::str::FromStr;

/// Does shit
pub trait FromScanf
where
    Self: Sized,
{
    /// Error type
    type Err: Error;
    /// Number of captures taken by this regex
    const NUM_CAPTURES: usize;
    /// Take shit and return shit
    fn from_matches(src: &mut regex::SubCaptureMatches) -> Result<Self, Self::Err>;
}

impl<T: FromStr> FromScanf for T
where
    <T as FromStr>::Err: Error,
{
    type Err = FromStrFailedError<T>;
    const NUM_CAPTURES: usize = 1;
    fn from_matches(src: &mut regex::SubCaptureMatches) -> Result<Self, Self::Err> {
        src.next()
            .expect("a")
            .expect("b")
            .as_str()
            .parse()
            .map_err(|error| FromStrFailedError {
                type_name: std::any::type_name::<T>(),
                error,
            })
    }
}

/// Error type for `FromScanf` impls for `FromStr`
pub struct FromStrFailedError<T: FromStr>
where
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
