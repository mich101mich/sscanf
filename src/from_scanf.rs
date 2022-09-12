use std::error::Error;
use std::str::FromStr;

use super::FromStrFailedError;

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

impl<T> FromScanf for T
where
    T: FromStr,
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
