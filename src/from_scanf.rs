use std::error::Error;
use std::str::FromStr;

use crate::FromStrFailedError;

/// A trait that allows you to use a custom regex for parsing a type.
///
/// **You should not implement this trait yourself.**
///
/// The recommended ways to implement this trait are:
/// * `#[derive(FromScanf)]`
/// * implement [`FromStr`] and use the [blanket implementation](#impl-FromScanf)
///
/// A manual implementation requires upholding the [`NUM_CAPTURES`](FromScanf::NUM_CAPTURES) invariant
/// yourself, which cannot be reliably checked and will lead to incomprehensible errors if violated.
///
/// ## Deriving
/// TODO:
///
/// ## Implementing [`FromStr`]
/// TODO:
pub trait FromScanf
where
    Self: Sized,
{
    /// Error type
    type Err: Error + 'static;
    /// Number of captures taken by this regex.
    ///
    /// **HAS** to match the number of unescaped capture groups in the [`RegexRepresentation`](crate::RegexRepresentation)
    /// +1 for the whole match.
    const NUM_CAPTURES: usize;
    /// The implementation of the parsing.
    ///
    /// **HAS** to take **EXACTLY** `NUM_CAPTURES` elements from the iterator.
    fn from_matches(src: &mut regex::SubCaptureMatches) -> Result<Self, Self::Err>;
}

impl<T> FromScanf for T
where
    T: FromStr + 'static,
    <T as FromStr>::Err: Error + 'static,
{
    type Err = FromStrFailedError<T>;
    const NUM_CAPTURES: usize = 1;
    fn from_matches(src: &mut regex::SubCaptureMatches) -> Result<Self, Self::Err> {
        src.next()
            .expect(crate::EXPECT_NEXT_HINT)
            .expect(crate::EXPECT_CAPTURE_HINT)
            .as_str()
            .parse()
            .map_err(FromStrFailedError::new)
    }
}
